/// The `font::Id` and `font::Map` types.
use std;
use rusttype;
use super::{Map, FontId};

/// The RustType `FontCollection` type used by conrod.
pub type FontCollection = rusttype::FontCollection<'static>;
/// The RustType `Font` type used by conrod.
pub type Font = rusttype::Font<'static>;

/// Returned when loading new fonts from file or bytes.
#[derive(Debug)]
pub enum Error {
    /// Some error occurred while loading a `FontCollection` from a file.
    IO(std::io::Error),
    /// No `Font`s could be yielded from the `FontCollection`.
    NoFont,
}

impl Map<FontId, Font> {
    /// Insert a single `Font` into the map by loading it from the given file path.
    pub fn insert_from_file<P>(&mut self, path: P) -> Result<FontId, Error>
        where P: AsRef<std::path::Path>
    {
        let font = try!(from_file(path));
        Ok(self.insert(font))
    }
}


/// Load a `FontCollection` from a file at a given path.
pub fn collection_from_file<P>(path: P) -> Result<FontCollection, std::io::Error>
    where P: AsRef<std::path::Path>
{
    use std::io::Read;
    let path = path.as_ref();
    let mut file = try!(std::fs::File::open(path));
    let mut file_buffer = Vec::new();
    try!(file.read_to_end(&mut file_buffer));
    Ok(FontCollection::from_bytes(file_buffer))
}

/// Load a single `Font` from a file at the given path.
pub fn from_file<P>(path: P) -> Result<Font, Error>
    where P: AsRef<std::path::Path>
{
    let collection = try!(collection_from_file(path));
    collection.into_font().ok_or(Error::NoFont)
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IO(e)
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::IO(ref e) => std::error::Error::description(e),
            Error::NoFont => "No `Font` found in the loaded `FontCollection`.",
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        writeln!(f, "{}", std::error::Error::description(self))
    }
}
