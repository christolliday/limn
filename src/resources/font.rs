//! Module for actions on a font. 
//! The underlying type for a font is 
use rusttype::{self, Font};
use errors::Error as LimnError;

fn load_font_data(name: &str) -> Result<Vec<u8>, LimnError> {
    use std::fs::File;
    use std::io::Read;
    let mut file = File::open(format!("assets/fonts/{}.ttf", name)).expect("Font missing");
    let mut data = Vec::new();
    try!(file.read_to_end(&mut data));
    Ok(data)
}

pub fn load_font(name: &str) -> Result<Font, > {
    let data = try!(load_font_data(name));
    let collection = rusttype::FontCollection::from_bytes(data);
    Ok(collection.into_font().unwrap())
}
