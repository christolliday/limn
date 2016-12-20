extern crate rusttype;
extern crate gfx;

use super::gfx::{G2d, G2dTexture};
use gfx_device_gl;
use gfx_graphics;
use texture;
use self::rusttype::gpu_cache::Cache;
use gfx_graphics::TextureSettings;

/// Glyph cache.
pub type Glyphs = gfx_graphics::GlyphCache<gfx_device_gl::Resources, gfx_device_gl::Factory>;

/// A wrapper around a `G2dTexture` and a rusttype GPU `Cache`
///
/// Using a wrapper simplifies the construction of both caches and ensures that they maintain
/// identical dimensions.
pub struct GlyphCache {
    pub cache: Cache,
    pub texture: G2dTexture<'static>,
    pub vertex_data: Vec<u8>,
}

impl GlyphCache {
    /// Constructor for a new `GlyphCache`.
    ///
    /// The `width` and `height` arguments are in pixel values.
    ///
    /// If you need to resize the `GlyphCache`, construct a new one and discard the old one.
    pub fn new(factory: &mut gfx_device_gl::Factory, width: u32, height: u32) -> Self {

        // Construct the rusttype GPU cache with the tolerances recommended by their documentation.
        const SCALE_TOLERANCE: f32 = 0.1;
        const POSITION_TOLERANCE: f32 = 0.1;
        let cache = Cache::new(width, height, SCALE_TOLERANCE, POSITION_TOLERANCE);

        // Construct a `G2dTexture`
        let buffer_len = width as usize * height as usize;
        let init = vec![128; buffer_len];
        let settings = TextureSettings::new();
        let texture = G2dTexture::from_memory_alpha(factory, &init, width, height, &settings).unwrap();

        GlyphCache {
            cache: cache,
            texture: texture,
            vertex_data: Vec::new(),
        }
    }
}

pub fn cache_queued_glyphs(graphics: &mut G2d,
                       cache: &mut G2dTexture<'static>,
                       rect: rusttype::Rect<u32>,
                       data: &[u8],
                       vertex_data: &mut Vec<u8>)
{
    use texture::UpdateTexture;

    // An iterator that efficiently maps the `byte`s yielded from `data` to `[r, g, b, byte]`;
    //
    // This is only used within the `cache_queued_glyphs` below, however due to a bug in rustc we
    // are unable to declare types inside the closure scope.
    struct Bytes { b: u8, i: u8 }
    impl Iterator for Bytes {
        type Item = u8;
        fn next(&mut self) -> Option<Self::Item> {
            let b = match self.i {
                0 => 255,
                1 => 255,
                2 => 255,
                3 => self.b,
                _ => return None,
            };
            self.i += 1;
            Some(b)
        }
    }

    let offset = [rect.min.x, rect.min.y];
    let size = [rect.width(), rect.height()];
    let format = texture::Format::Rgba8;
    let encoder = &mut graphics.encoder;

    vertex_data.clear();
    vertex_data.extend(data.iter().flat_map(|&b| Bytes { b: b, i: 0 }));
    UpdateTexture::update(cache, encoder, format, &vertex_data[..], offset, size)
        .expect("Failed to update texture");
}


