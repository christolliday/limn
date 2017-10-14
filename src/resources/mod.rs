#[macro_use]
pub mod id;

use std::sync::{Mutex, MutexGuard};
use std::collections::HashMap;

use webrender::api::*;
use image;
use rusttype;
use app_units;
use font_loader::system_fonts;

use text_layout;

use self::id::{Id, IdGen};

pub type Font = rusttype::Font<'static>;

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

pub struct FontInfo {
    pub key: FontKey,
    pub info: Font,
}

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
    pub fn new() -> Self {
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

    pub fn get_image(&mut self, name: &str) -> &ImageInfo {
        if self.images.contains_key(name) {
            &self.images[name]
        } else {
            let (data, descriptor) = load_image(name).unwrap();
            self.put_image(name, data, descriptor)
        }
    }

    pub fn put_image(&mut self, name: &str, data: ImageData, descriptor: ImageDescriptor) -> &ImageInfo {
        let key = self.render.as_ref().unwrap().generate_image_key();
        let mut resources = ResourceUpdates::new();
        resources.add_image(key, descriptor, data, None);
        self.render.as_ref().unwrap().update_resources(resources);
        let image_info = ImageInfo { key: key, info: descriptor };
        self.images.insert(name.to_owned(), image_info);
        &self.images[name]
    }

    pub fn update_image(&mut self, name: &str, data: ImageData, descriptor: ImageDescriptor) -> &ImageInfo {
        let mut image_info = self.images.remove(name).expect("update image");
        let mut resources = ResourceUpdates::new();
        resources.update_image(image_info.key, descriptor, data, None);
        self.render.as_ref().unwrap().update_resources(resources);
        image_info.info = descriptor;
        self.images.insert(name.to_owned(), image_info);
        &self.images[name]
    }

    #[deprecated(note = "may panic, instead of this use get_font_or_load_from_system or get_font_if_present")]
    pub fn get_font(&mut self, name: &str) -> &FontInfo {
        self.get_font_or_load_from_system(name).unwrap()
    }

    pub fn get_font_if_present(&mut self, name: &str) -> Option<&FontInfo> {
        self.fonts.get(name)
    }

    pub fn get_font_or_load_from_system(&mut self, name: &str) -> Result<&FontInfo, ::std::io::Error> {
        if !self.fonts.contains_key(name) {
            return self.add_font(name, try!(load_system_font_by_family_name(name)));
        }
        Ok(&self.fonts[name])
    }

    pub fn add_font(&mut self, name: &str, font_bytes: Vec<u8>) -> Result<&FontInfo, ::std::io::Error> {
        let font = try!(font_from_bytes(font_bytes.clone()));

        let key = self.render.as_ref().unwrap().generate_font_key();
        let mut resources = ResourceUpdates::new();
        resources.add_raw_font(key, font_bytes, 0);

        self.render.as_ref().unwrap().update_resources(resources);
        let font_info = FontInfo { key: key, info: font };
        self.fonts.insert(name.to_owned(), font_info);

        Ok(&self.fonts[name])
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

fn load_system_font_by_family_name(name: &str) -> Result<Vec<u8>, ::std::io::Error> {
    let property = system_fonts::FontPropertyBuilder::new().family(name).build();
    let font = system_fonts::get(&property)
        .map(|tuple| tuple.0)
        .ok_or(::std::io::Error::new(::std::io::ErrorKind::NotFound, "Font not found"));
    font
}

fn font_from_bytes(bytes: Vec<u8>) -> Result<Font, ::std::io::Error> {
    let collection = rusttype::FontCollection::from_bytes(bytes);
    let mut font_iter = collection.into_fonts();
    font_iter.next().ok_or(::std::io::Error::new(::std::io::ErrorKind::InvalidData, "Bad font format"))
}

pub fn load_font(name: &str) -> Result<Font, ::std::io::Error> {
    font_from_bytes(try!(load_system_font_by_family_name(name)))
}
