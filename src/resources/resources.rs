use std::sync::{Mutex, MutexGuard};
use std::collections::HashMap;
use webrender::api::*;

use app_units;
use text_layout;
use id::{Id, IdGen};

lazy_static! {
    static ref RES: Mutex<Resources> = Mutex::new(Resources::new());
}

pub fn init_resources(render_api: RenderApi) {
    RES.try_lock().unwrap().render = Some(render_api);
}

// Allow global access to Resources
pub fn resources() -> MutexGuard<'static, Resources> {
    RES.try_lock().unwrap()
}

named_id!(WidgetId);

#[derive(Clone)]
pub struct ImageInfo {
    pub key: ImageKey,
    pub info: ImageDescriptor,
}

/// Map for a given `Id` and resource type.
pub struct Map<I, T> {
    id_gen: IdGen<I>,
    map: HashMap<I, T>,
}

impl<I: Id, T> Map<I, T> {

    pub fn new() -> Self {
        Map {
            id_gen: IdGen::new(),
            map: HashMap::new(),
        }
    }
    
    /// Borrow the resource associated with the given `Id`.
    pub fn get(&self, id: I) -> Option<&T> {
        self.map.get(&id)
    }
    
    /// Borrow the resource associated with the given `Id`.
    pub fn get_mut(&self, id: I) -> Option<&T> {
        self.map.get_mut(&id)
    }
    
    /// Adds the given resource to the `Map` and returns a unique `Id` for it.
    pub fn insert(&mut self, resource: T) -> I {
        let id = self.id_gen.next();
        self.map.insert(id, resource);
        id
    }
}

pub struct Resources {
    pub render: Option<RenderApi>,
    pub fonts: HashMap<String, FontInfo>,
    pub font_instances: HashMap<(String, app_units::Au), FontInstanceKey>,
    pub images: HashMap<String, ImageInfo>,
    pub texture_descriptors: HashMap<u64, ImageDescriptor>,
    pub widget_id: IdGen<WidgetId>,
}

impl Resources {
    /// This function only gets called to instantiate a singleton-like resource container
    /// It should not be called ouside of limn, which is why it's not public
    fn new() -> Self {
        Resources {
            render: None,
            fonts: HashMap::new(),
            font_instances: HashMap::new(),
            images: HashMap::new(),
            texture_descriptors: HashMap::new(),
            widget_id: IdGen::new(),
        }
    }
    pub fn widget_id(&mut self) -> WidgetId {
        self.widget_id.next()
    }

    pub fn get_font_instance(&mut self, name: &str, font_size: f32) -> &FontInstanceKey {
        let font_key = self.get_font(name).key;
        let size = app_units::Au::from_f32_px(text_layout::px_to_pt(font_size));
        if !self.font_instances.contains_key(&(name.to_owned(), size)) {
            let instance_key = self.render.as_ref().unwrap().generate_font_instance_key();
            let mut resources = ResourceUpdates::new();
            resources.add_font_instance(instance_key, font_key, size, None, None, Vec::new());
            self.render.as_ref().unwrap().update_resources(resources);
            self.font_instances.insert((name.to_owned(), size), instance_key);
        }
        &self.font_instances[&(name.to_owned(), size)]
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
        ImageFormat::RGB8 => true,
        ImageFormat::RG8 => true,
        ImageFormat::A8 => false,
        ImageFormat::Invalid | ImageFormat::RGBAF32 => unreachable!(),
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

fn load_font_data(name: &str) -> Result<Vec<u8>, ::std::io::Error> {
    use std::fs::File;
    use std::io::Read;
    let mut file = File::open(format!("assets/fonts/{}.ttf", name)).expect("Font missing");
    let mut data = Vec::new();
    try!(file.read_to_end(&mut data));
    Ok(data)
}

pub fn load_font(name: &str) -> Result<Font, ::std::io::Error> {
    let data = try!(load_font_data(name));
    let collection = rusttype::FontCollection::from_bytes(data);
    Ok(collection.into_font().unwrap())
}
