use std::{error, io, fmt};
use std::path::Path;
use std::fs::File;
use std::fmt::{Formatter, Display};

use rusttype;
use super::{Map, FontId};

pub type FontCollection = rusttype::FontCollection<'static>;
pub type Font = rusttype::Font<'static>;

/// Returned when loading new fonts from file or bytes.
#[derive(Debug)]
pub enum Error {
    /// Some error occurred while loading a `FontCollection` from a file.
    IO(io::Error),
    /// No `Font`s could be yielded from the `FontCollection`.
    NoFont,
}

impl Map<FontId, Font> {
    /// Insert a single `Font` into the map by loading it from the given file path.
    pub fn insert_from_file<P>(&mut self, path: P) -> Result<FontId, Error>
        where P: AsRef<Path>
    {
        let font = try!(from_file(path));
        Ok(self.insert(font))
    }
}


/// Load a `FontCollection` from a file at a given path.
pub fn collection_from_file<P>(path: P) -> Result<FontCollection, io::Error>
    where P: AsRef<Path>
{
    use std::io::Read;
    let path = path.as_ref();
    let mut file = try!(File::open(path));
    let mut file_buffer = Vec::new();
    try!(file.read_to_end(&mut file_buffer));
    Ok(FontCollection::from_bytes(file_buffer))
}

/// Load a single `Font` from a file at the given path.
pub fn from_file<P>(path: P) -> Result<Font, Error>
    where P: AsRef<Path>
{
    let collection = try!(collection_from_file(path));
    collection.into_font().ok_or(Error::NoFont)
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::IO(e)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::IO(ref e) => error::Error::description(e),
            Error::NoFont => "No `Font` found in the loaded `FontCollection`.",
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        writeln!(f, "{}", error::Error::description(self))
    }
}
