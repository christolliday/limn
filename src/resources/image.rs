
use gfx_device_gl::Factory;
use gfx_graphics::{TextureSettings, Flip};
use resources::{Map, Id};
use std::path::Path;
use backend::gfx::G2dTexture;

pub type Texture = G2dTexture<'static>;

impl Map<Texture> {
    pub fn insert_from_file<P>(&mut self, factory: &mut Factory, path: P) -> Id
        where P: AsRef<Path>
    {
        let settings = TextureSettings::new();
        let image = Texture::from_path(factory, &path, Flip::None, &settings).unwrap();
        self.insert(image)
    }
}
