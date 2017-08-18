use std::collections::HashMap;

use gleam::gl;
use glutin;
use webrender;
use webrender::api::*;
use stb_truetype;
use image;

use window::Window;
//use util::Point;
use euclid::TypedPoint2D;

pub struct WebRenderContext {
    renderer: webrender::Renderer,
    pub render_api: RenderApi,
    pub epoch: Epoch,
    pub pipeline_id: PipelineId,
    pub document_id: DocumentId,
    pub root_background_color: ColorF,
    pub fonts: HashMap<String, FontInfo>,
    pub images: HashMap<String, ImageInfo>,
}

pub struct RenderBuilder<'a> {
    pub render: &'a mut WebRenderContext,
    pub builder: DisplayListBuilder,
    pub resources: ResourceUpdates,
}

pub struct FontInfo {
    pub key: FontKey,
    pub info: stb_truetype::FontInfo<Vec<u8>>,
}

pub struct ImageInfo {
    pub key: ImageKey,
    pub info: ImageDescriptor,
}

impl <'a> RenderBuilder<'a> {
    pub fn get_image(&mut self, name: &str) -> &ImageInfo {
        if !self.render.images.contains_key(name) {
            let (data, descriptor) = load_image(name).unwrap();
            let key = self.render.render_api.generate_image_key();
            self.resources.add_image(key, descriptor, data, None);
            let image_info = ImageInfo { key: key, info: descriptor };
            self.render.images.insert(name.to_owned(), image_info);
        }
        &self.render.images[name]
    }
    pub fn get_font(&mut self, name: &str) -> &FontInfo {
        if !self.render.fonts.contains_key(name) {
            let data = load_font(name).unwrap();
            let key = self.render.render_api.generate_font_key();
            let info = stb_truetype::FontInfo::new(data.clone(), 0).unwrap();
            self.resources.add_raw_font(key, data, 0);
            let font_info = FontInfo { key: key, info: info };
            self.render.fonts.insert(name.to_owned(), font_info);
        }
        &self.render.fonts[name]
    }
}

// From webrender/wrench
// These are slow. Gecko's gfx/2d/Swizzle.cpp has better versions
pub fn premultiply(data: &mut [u8]) {
    for pixel in data.chunks_mut(4) {
        let a = pixel[3] as u32;
        let r = pixel[2] as u32;
        let g = pixel[1] as u32;
        let b = pixel[0] as u32;

        pixel[3] = a as u8;
        pixel[2] = ((r * a + 128) / 255) as u8;
        pixel[1] = ((g * a + 128) / 255) as u8;
        pixel[0] = ((b * a + 128) / 255) as u8;
    }
}

fn load_image(file: &str) -> Result<(ImageData, ImageDescriptor), image::ImageError> {
    use image::GenericImage;
    let image = try!(image::open(format!("assets/images/{}", file)));
    let image_dims = image.dimensions();
    let format = match image {
        image::ImageLuma8(_) => ImageFormat::A8,
        image::ImageRgb8(_) => ImageFormat::RGB8,
        image::ImageRgba8(_) => ImageFormat::BGRA8,
        image::ImageLumaA8(_) => {
            return Err(image::ImageError::UnsupportedError(format!("ImageLumaA8 unsupported")));
        }
    };
    let mut bytes = image.raw_pixels();
    if format == ImageFormat::BGRA8 {
        premultiply(bytes.as_mut_slice());
    }
    let opaque = is_image_opaque(format, &bytes[..]);
    let descriptor = ImageDescriptor::new(image_dims.0, image_dims.1, format, opaque);
    let data = ImageData::new(bytes);
    Ok((data, descriptor))
}
fn load_font(name: &str) -> Result<Vec<u8>, ::std::io::Error> {
    use std::fs::File;
    use std::io::Read;
    let mut file = File::open(format!("assets/fonts/{}.ttf", name)).expect("Font missing");
    let mut data = Vec::new();
    try!(file.read_to_end(&mut data));
    Ok(data)
}
fn is_image_opaque(format: ImageFormat, bytes: &[u8]) -> bool {
    match format {
        ImageFormat::BGRA8 => {
            let mut is_opaque = true;
            for i in 0..(bytes.len() / 4) {
                if bytes[i * 4 + 3] != 255 {
                    is_opaque = false;
                    break;
                }
            }
            is_opaque
        }
        ImageFormat::RGB8 => true,
        ImageFormat::RG8 => true,
        ImageFormat::A8 => false,
        ImageFormat::Invalid | ImageFormat::RGBAF32 => unreachable!(),
    }
}

impl WebRenderContext {
    pub fn new(window: &mut Window, events_loop: &glutin::EventsLoop) -> Self {
        let gl = window.gl();
        println!("OpenGL version {}", gl.get_string(gl::VERSION));

        let opts = webrender::RendererOptions {
            resource_override_path: None,
            debug: true,
            precache_shaders: false,
            device_pixel_ratio: window.hidpi_factor(),
            .. webrender::RendererOptions::default()
        };

        let (renderer, sender) = webrender::renderer::Renderer::new(gl, opts).unwrap();
        let api = sender.create_api();
        let document_id = api.add_document(window.size_u32());

        let notifier = Box::new(Notifier { events_proxy: events_loop.create_proxy() });
        renderer.set_render_notifier(notifier);

        let epoch = Epoch(0);
        let root_background_color = ColorF::new(0.8, 0.8, 0.8, 1.0);

        let pipeline_id = PipelineId(0, 0);
        api.set_root_pipeline(document_id, pipeline_id);
        WebRenderContext {
            renderer: renderer,
            render_api: api,
            epoch: epoch,
            pipeline_id: pipeline_id,
            document_id: document_id,
            root_background_color: root_background_color,
            fonts: HashMap::new(),
            images: HashMap::new(),
        }
    }
    pub fn deinit(self) {
        self.renderer.deinit();
    }
    pub fn render_builder(&mut self, window_size: LayoutSize) -> RenderBuilder {
        let builder = DisplayListBuilder::new(self.pipeline_id, window_size);
        RenderBuilder {
            render: self,
            builder: builder,
            resources: ResourceUpdates::new(),
        }
    }
    pub fn set_display_list(&mut self, builder: DisplayListBuilder, resources: ResourceUpdates, window_size: LayoutSize) {
        self.render_api.set_display_list(
            self.document_id,
            self.epoch,
            Some(self.root_background_color),
            window_size,
            builder.finalize(),
            true,
            resources
        );
    }
    pub fn generate_frame(&mut self) {
        self.render_api.generate_frame(self.document_id, None);
    }
    // if there is a frame ready, update current frame and render it, otherwise, does nothing
    pub fn update(&mut self, window_size: DeviceUintSize) {
        self.renderer.update();
        self.renderer.render(window_size);
    }
    pub fn toggle_flags(&mut self, toggle_flags: webrender::renderer::DebugFlags) {
        let mut flags = self.renderer.get_debug_flags();
        flags.toggle(toggle_flags);
        self.renderer.set_debug_flags(flags);
    }
    pub fn window_resized(&mut self, size: DeviceUintSize) {
        let window_rect = DeviceUintRect::new(TypedPoint2D::zero(), size);
        self.render_api.set_window_parameters(self.document_id, size, window_rect);
    }
}

struct Notifier {
    events_proxy: glutin::EventsLoopProxy,
}
impl Notifier {
    fn new(events_proxy: glutin::EventsLoopProxy) -> Self {
        Notifier {
            events_proxy: events_proxy,
        }
    }
}

impl RenderNotifier for Notifier {
    fn new_frame_ready(&mut self) {
        #[cfg(not(target_os = "android"))]
        self.events_proxy.wakeup().ok();
    }

    fn new_scroll_frame_ready(&mut self, _composite_needed: bool) {
        #[cfg(not(target_os = "android"))]
        self.events_proxy.wakeup().ok();
    }
}
