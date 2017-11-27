use std::collections::HashMap;

use rusttype;
use font_loader::system_fonts;
use app_units;
use webrender::api::{RenderApi, ResourceUpdates, FontKey, FontInstanceKey};

use text_layout;

pub type Font = rusttype::Font<'static>;

pub struct FontInfo {
    pub key: FontKey,
    pub info: Font,
}

#[derive(Default)]
pub struct FontLoader {
    pub render: Option<RenderApi>,
    pub fonts: HashMap<String, FontInfo>,
    pub font_instances: HashMap<(String, app_units::Au), FontInstanceKey>,
}

impl FontLoader {
    pub fn new() -> Self {
        FontLoader::default()
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
