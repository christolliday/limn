#[macro_use]
pub mod id;
pub mod font;

use std::sync::{Mutex, MutexGuard};
use std::collections::HashMap;
use std::default::Default;

use webrender::api::*;
use image;

use self::id::{Id, IdGen};
use self::font::FontLoader;

use style::Theme;

lazy_static! {
    static ref RES: Mutex<Resources> = Mutex::new(Resources::new());
}

pub fn init_resources(render_api: RenderApiSender) {
    RES.try_lock().unwrap().set_render_api(render_api);
}
// Allow global access to Resources
pub fn resources() -> MutexGuard<'static, Resources> {
    RES.try_lock().unwrap()
}

named_id!(WidgetId);


#[derive(Debug, Clone)]
pub struct ImageInfo {
    pub key: ImageKey,
    pub info: ImageDescriptor,
}

/// Map for a given `Id` and resource type.
pub struct Map<I, T> {
    id_gen: IdGen<I>,
    map: HashMap<I, T>,
}

impl<I: Id, T> Default for Map<I, T> {
    #[inline]
    fn default() -> Self {
        Map {
            id_gen: IdGen::new(),
            map: HashMap::new(),
        }
    }
}
impl<I: Id, T> Map<I, T> {

    pub fn new() -> Self {
        Self::default()
    }

    /// Borrow the resource associated with the given `Id`.
    pub fn get(&self, id: I) -> Option<&T> {
        self.map.get(&id)
    }
    /// Adds the given resource to the `Map` and returns a unique `Id` for it.
    pub fn insert(&mut self, resource: T) -> I {
        let id = self.id_gen.next_id();
        self.map.insert(id, resource);
        id
    }
}

pub struct Resources {
    pub render: Option<RenderApi>,
    pub font_loader: FontLoader,
    pub images: HashMap<String, ImageInfo>,
    pub texture_descriptors: HashMap<u64, ImageDescriptor>,
    pub widget_id: IdGen<WidgetId>,
    pub theme: Theme,
}

impl Default for Resources {
    fn default() -> Self {
        Resources {
            render: None,
            font_loader: FontLoader::new(),
            images: HashMap::new(),
            texture_descriptors: HashMap::new(),
            widget_id: IdGen::new(),
            theme: Theme::new(),
        }
    }
}

impl Resources {
    /// Creates a new `Resources` struct, same as calling `default()`
    pub fn new() -> Self {
        Self::default()
    }

    pub fn widget_id(&mut self) -> WidgetId {
        self.widget_id.next_id()
    }

    pub fn get_image(&mut self, name: &str) -> &ImageInfo {
        if self.images.contains_key(name) {
            &self.images[name]
        } else {
            let (data, descriptor) = load_image(name).unwrap();
            self.put_image(name, data, descriptor)
        }
    }

    pub fn put_image(&mut self, name: &str, data: ImageData, descriptor: ImageDescriptor) -> &ImageInfo {
        let key = self.render_api().generate_image_key();
        let mut resources = ResourceUpdates::new();
        resources.add_image(key, descriptor, data, None);
        self.render_api().update_resources(resources);
        let image_info = ImageInfo { key: key, info: descriptor };
        self.images.insert(name.to_owned(), image_info);
        &self.images[name]
    }

    pub fn create_texture(&mut self, data: ImageData, descriptor: ImageDescriptor) -> ImageInfo {
        let key = self.render_api().generate_image_key();
        let mut resources = ResourceUpdates::new();
        resources.add_image(key, descriptor, data, None);
        self.render_api().update_resources(resources);
        ImageInfo { key: key, info: descriptor }
    }

    fn set_render_api(&mut self, render: RenderApiSender) {
        self.render = Some(render.create_api());
        self.font_loader.render = Some(render.create_api());
    }

    pub fn render_api(&self) -> &RenderApi {
        self.render.as_ref().unwrap()
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
            return Err(image::ImageError::UnsupportedError("ImageLumaA8 unsupported".to_string()));
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
        ImageFormat::RGB8 | ImageFormat::RG8 => true,
        ImageFormat::A8 => false,
        ImageFormat::Invalid | ImageFormat::RGBAF32 => unreachable!(),
    }
}

// From webrender/wrench
// These are slow. Gecko's gfx/2d/Swizzle.cpp has better versions
pub fn premultiply(data: &mut [u8]) {
    for pixel in data.chunks_mut(4) {
        let a = u32::from(pixel[3]);
        let r = u32::from(pixel[2]);
        let g = u32::from(pixel[1]);
        let b = u32::from(pixel[0]);

        pixel[3] = a as u8;
        pixel[2] = ((r * a + 128) / 255) as u8;
        pixel[1] = ((g * a + 128) / 255) as u8;
        pixel[0] = ((b * a + 128) / 255) as u8;
    }
}
