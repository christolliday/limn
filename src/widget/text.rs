use util::*;
use graphics;
use graphics::Context;
use backend::glyph;
use super::super::text::{self, Wrap};
use super::super::resources;
use super::super::ui::Resources;
use backend::glyph::GlyphCache;
use backend::gfx::G2d;
use backend::gfx::ImageSize;
use graphics::types::Color;
use std::any::Any;

pub struct TextDrawable {
    pub text: String,
    pub font_id: resources::Id,
    pub font_size: Scalar,
    pub text_color: Color,
    pub background_color: Color,
}
impl TextDrawable {
    pub fn new(text: String, font_id: resources::Id) -> Self {
        TextDrawable {
            text: text,
            font_id: font_id,
            font_size: 24.0,
            text_color: [0.0, 0.0, 0.0, 1.0],
            background_color: [1.0, 1.0, 1.0, 1.0],
        }
    }
    pub fn measure_dims_no_wrap(&self, resources: &Resources) -> Dimensions {
        let font = resources.fonts.get(self.font_id).unwrap();
        text::get_text_dimensions(&self.text,
                                  font,
                                  self.font_size,
                                  self.font_size * 1.25,
                                  Align::Start,
                                  Align::Start)
    }
    pub fn measure_height_wrapped(&self, resources: &Resources, width: Scalar) -> Scalar {
        let font = resources.fonts.get(self.font_id).unwrap();
        text::get_text_height(&self.text,
                              font,
                              self.font_size,
                              self.font_size * 1.25,
                              width,
                              Wrap::Character,
                              Align::Start,
                              Align::Start)
    }
}

pub fn draw_text(state: &Any,
                 parent_bounds: Rectangle,
                 bounds: Rectangle,
                 resources: &mut Resources,
                 context: Context,
                 graphics: &mut G2d) {
    let state: &TextDrawable = state.downcast_ref().unwrap();

    graphics::Rectangle::new(state.background_color)
        .draw(bounds, &context.draw_state, context.transform, graphics);

    let GlyphCache { texture: ref mut text_texture_cache,
                     cache: ref mut glyph_cache,
                     ref mut vertex_data } = resources.glyph_cache;

    let font = resources.fonts.get(state.font_id).unwrap();
    let line_wrap = Wrap::Character;

    let positioned_glyphs = &text::get_positioned_glyphs(&state.text,
                                                         bounds,
                                                         font,
                                                         state.font_size,
                                                         state.font_size * 1.25,
                                                         line_wrap,
                                                         Align::Start,
                                                         Align::Start);

    // Queue the glyphs to be cached.
    for glyph in positioned_glyphs.iter() {
        glyph_cache.queue_glyph(state.font_id.index(), glyph.clone());
    }

    // Cache the glyphs within the GPU cache.
    glyph_cache.cache_queued(|rect, data| {
            glyph::cache_queued_glyphs(graphics, text_texture_cache, rect, data, vertex_data)
        })
        .unwrap();

    let tex_dim = {
        let (tex_w, tex_h) = text_texture_cache.get_size();
        Dimensions {
            width: tex_w as f64,
            height: tex_h as f64,
        }
    };

    let rectangles = positioned_glyphs.into_iter()
        .filter_map(|g| glyph_cache.rect_for(state.font_id.index(), g).ok().unwrap_or(None))
        .map(|(uv_rect, screen_rect)| (map_rect_i32(screen_rect), map_rect_f32(uv_rect) * tex_dim));
    // A re-usable buffer of rectangles describing the glyph's screen and texture positions.
    let mut glyph_rectangles = Vec::new();
    glyph_rectangles.extend(rectangles);
    graphics::image::draw_many(&glyph_rectangles,
                               state.text_color,
                               text_texture_cache,
                               &context.draw_state,
                               context.transform,
                               graphics);
}
