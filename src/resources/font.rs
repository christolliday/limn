//! Module for actions on a font. 
//! The underlying type for a font is 
use rusttype::{self, Font};


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
