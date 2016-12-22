use util::*;
use graphics;
use graphics::Context;
use backend::glyph;
use super::super::text::{self, Wrap, font};
use super::super::ui::Resources;
use backend::glyph::GlyphCache;
use backend::gfx::G2d;
use super::WidgetDrawable;
use backend::gfx::ImageSize;
use graphics::types::Color;

pub struct TextDrawable {
    pub text: String,
    pub font_id: font::Id,
    pub font_size: Scalar,
    pub text_color: Color,
    pub background_color: Color,
}

impl WidgetDrawable for TextDrawable {
    fn draw(&self,
            bounds: Rectangle,
            resources: &mut Resources,
            context: Context,
            graphics: &mut G2d) {

        graphics::Rectangle::new(self.background_color)
            .draw(bounds, &context.draw_state, context.transform, graphics);

        let GlyphCache { texture: ref mut text_texture_cache,
                         cache: ref mut glyph_cache,
                         ref mut vertex_data } = resources.glyph_cache;

        let font = resources.fonts.get(self.font_id).unwrap();
        let line_wrap = Wrap::Character;

        let positioned_glyphs = &text::get_positioned_glyphs(&self.text,
                                                             bounds,
                                                             font,
                                                             self.font_size,
                                                             line_wrap,
                                                             Align::Start,
                                                             Align::Start);

        // Queue the glyphs to be cached.
        for glyph in positioned_glyphs.iter() {
            glyph_cache.queue_glyph(self.font_id.index(), glyph.clone());
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
            .filter_map(|g| glyph_cache.rect_for(self.font_id.index(), g).ok().unwrap_or(None))
            .map(|(uv_rect, screen_rect)| {
                (map_rect_i32(screen_rect), map_rect_f32(uv_rect) * tex_dim)
            });
        // A re-usable buffer of rectangles describing the glyph's screen and texture positions.
        let mut glyph_rectangles = Vec::new();
        glyph_rectangles.extend(rectangles);
        graphics::image::draw_many(&glyph_rectangles,
                                   self.text_color,
                                   text_texture_cache,
                                   &context.draw_state,
                                   context.transform,
                                   graphics);
    }
}