//! Module for actions on a font.
//! The underlying type for a font is a `rusttype::Font`

use rusttype;
use resources::errors::Error as LimnResourcesError;
use resources::errors::ErrorKind as LimnResourcesErrorKind;
use std::path::Path;
use std::io::prelude::*;

pub struct Font(::rusttype::Font<'static>);

impl Font {

    /// Convenience function, load a font from a file
    pub fn from_file<P: AsRef<Path>>(path: P)
        -> Result<Self, LimnResourcesError>
    {
        use std::fs::File;
        let mut file = File::open(path)?;
        Self::try_from(&mut file)
    }

    /// Convenience function for loading fonts from system fonts
    #[cfg(feature = "font_loader")]
    pub fn from_font_loader(family_name: &str)
        -> Result<Self, LimnResourcesError>
    {
        use std::io::Cursor;
        let property = system_fonts::FontPropertyBuilder::new().family(family_name).build();
        let font = system_fonts::get(&property)
            .map(|tuple| tuple.0).ok()?;
        Self::try_from(&mut Cursor::new(font))
    }

    /// Create fonts from any source that implements `Read`
    pub fn try_from<R: Read>(source: &mut R)
    -> Result<Self, LimnResourcesError>
    {
        let mut buf = Vec::new();
        source.read_to_end(&mut buf)?;
        let collection = rusttype::FontCollection::from_bytes(buf);
        let font = collection.into_font()
                  .ok_or(LimnResourcesErrorKind::EmptyFontCollection)?;

        Ok(Self { 0: font })
    }
}
