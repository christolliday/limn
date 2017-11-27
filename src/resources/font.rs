use std::collections::HashMap;
use std::io;

use rusttype;
use font_loader::system_fonts::{self, FontProperty, FontPropertyBuilder};
use app_units;
use webrender::api::{RenderApi, ResourceUpdates, FontKey, FontInstanceKey};

use text_layout;

pub type Font = rusttype::Font<'static>;

pub struct FontInfo {
    pub key: FontKey,
    pub info: Font,
}

/// Set of properties used to specify a font
#[derive(Default, PartialEq, Eq, Hash, Clone, Debug)]
pub struct FontDescriptor {
    pub family_name: String,
    pub italic: bool,
    pub bold: bool,
}

impl FontDescriptor {
    pub fn from_family(family_name: &str) -> Self {
        FontDescriptor {
            family_name: String::from(family_name),
            ..FontDescriptor::default()
        }
    }
    fn property(&self) -> FontProperty {
        let mut builder = FontPropertyBuilder::new().family(&self.family_name);
        if self.italic {
            builder = builder.italic();
        }
        if self.bold {
            builder = builder.bold();
        }
        builder.build()
    }
}

#[derive(Default)]
pub struct FontLoader {
    pub render: Option<RenderApi>,
    pub font_info: HashMap<FontDescriptor, FontInfo>,
    pub bundled_font_info: HashMap<FontDescriptor, FontInfo>,
    pub font_instances: HashMap<(FontDescriptor, app_units::Au), FontInstanceKey>,
}

impl FontLoader {
    pub fn new() -> Self {
        FontLoader::default()
    }

    pub fn get_font(&mut self, descriptor: &FontDescriptor) -> &FontInfo {
        self.get_font_info(descriptor).unwrap()
    }

    fn get_font_info(&mut self, descriptor: &FontDescriptor) -> Result<&FontInfo, io::Error> {
        if self.bundled_font_info.contains_key(descriptor) {
            Ok(&self.bundled_font_info[descriptor])
        } else {
            if !self.font_info.contains_key(descriptor) {
                if let Ok(data) = system_fonts_load_data(&descriptor.property()) {
                    let font_info = self.load_font(data)?;
                    self.font_info.insert(descriptor.clone(), font_info);
                } else {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "No system font found"));
                }
            }
            Ok(&self.font_info[descriptor])
        }
    }

    pub fn get_font_instance(&mut self, descriptor: &FontDescriptor, font_size: f32) -> &FontInstanceKey {
        let font_key = self.get_font(descriptor).key;
        let size = app_units::Au::from_f32_px(text_layout::px_to_pt(font_size));
        let key = (descriptor.clone(), size);
        if !self.font_instances.contains_key(&key) {
            let instance_key = webrender_load_font_instance(self.render_api(), font_key, size);
            self.font_instances.insert(key.clone(), instance_key);
        }
        &self.font_instances[&key]
    }

    fn load_font(&mut self, data: Vec<u8>) -> Result<FontInfo, io::Error> {
        let font_info = rusttype_load_font_info(data.clone())?;
        let key = webrender_load_font(self.render_api(), data)?;
        Ok(FontInfo { key: key, info: font_info })
    }

    pub fn register_font_data(&mut self, descriptor: FontDescriptor, data: Vec<u8>) {
        let info = self.load_font(data).unwrap();
        self.bundled_font_info.insert(descriptor.clone(), info);
    }

    fn render_api(&self) -> &RenderApi {
        self.render.as_ref().unwrap()
    }
}

fn webrender_load_font(render_api: &RenderApi, data: Vec<u8>) -> Result<FontKey, io::Error> {
    let key = render_api.generate_font_key();
    let mut resources = ResourceUpdates::new();
    resources.add_raw_font(key, data, 0);
    render_api.update_resources(resources);
    Ok(key)
}

fn webrender_load_font_instance(render_api: &RenderApi, font_key: FontKey, size: app_units::Au) -> FontInstanceKey {
    let instance_key = render_api.generate_font_instance_key();
    let mut resources = ResourceUpdates::new();
    resources.add_font_instance(instance_key, font_key, size, None, None, Vec::new());
    render_api.update_resources(resources);
    instance_key
}

fn system_fonts_load_data(property: &FontProperty) -> Result<Vec<u8>, io::Error> {
    let font = system_fonts::get(&property)
        .map(|tuple| tuple.0)
        .ok_or(io::Error::new(io::ErrorKind::NotFound, "Font not found"));
    font
}

/// Read font data to get font information, v_metrics, glyph info etc.
fn rusttype_load_font_info(data: Vec<u8>) -> Result<Font, io::Error> {
    let collection = rusttype::FontCollection::from_bytes(data);
    let mut font_iter = collection.into_fonts();
    font_iter.next().ok_or(io::Error::new(io::ErrorKind::InvalidData, "Bad font format"))
}
