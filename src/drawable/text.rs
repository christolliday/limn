use graphics;
use graphics::types::Color;
use graphics::Context;

use backend::glyph::{self, GlyphCache};
use backend::gfx::{ImageSize, G2d};

use text_layout::{self, Wrap, Align};
use resources::{FontId, resources};
use util::{self, Dimensions, Scalar, Rectangle};
use widget::drawable::Drawable;
use widget::property::PropSet;
use widget::style::{self, Value, StyleField};
use color::*;

pub struct TextDrawable {
    pub text: String,
    pub font_id: FontId,
    pub font_size: Scalar,
    pub text_color: Color,
    pub background_color: Color,
    pub wrap: Wrap,
    pub align: Align,
}
impl Default for TextDrawable {
    fn default() -> Self {
        TextDrawable {
            text: "".to_owned(),
            font_id: FontId(0),
            font_size: 20.0,
            text_color: BLACK,
            background_color: TRANSPARENT,
            wrap: Wrap::Whitespace,
            align: Align::Start,
        }
    }
}

impl TextDrawable {
    pub fn new() -> Self {
        TextDrawable::default()
    }
    pub fn measure(&self) -> Dimensions {
        let res = resources();
        let font = res.fonts.get(self.font_id).unwrap();
        text_layout::get_text_dimensions(&self.text,
                                         font,
                                         self.font_size,
                                         self.font_size * 1.25,
                                         self.wrap)
            .into()
    }
}

impl Drawable for TextDrawable {
    fn draw(&mut self, bounds: Rectangle, crop_to: Rectangle, glyph_cache: &mut GlyphCache, context: Context, graphics: &mut G2d) {
        graphics::Rectangle::new(self.background_color)
                .draw(bounds, &context.draw_state, context.transform, graphics);

            let &mut GlyphCache { texture: ref mut text_texture_cache,
                                cache: ref mut glyph_cache,
                                ref mut vertex_data } = glyph_cache;

            let res = resources();
            let font = res.fonts.get(self.font_id).unwrap();

            let positioned_glyphs = &text_layout::get_positioned_glyphs(&self.text,
                                                                        bounds.into(),
                                                                        font,
                                                                        self.font_size,
                                                                        self.font_size * 1.25,
                                                                        self.wrap,
                                                                        self.align,
                                                                        Align::Start);

            // Queue the glyphs to be cached.
            for glyph in positioned_glyphs.iter() {
                glyph_cache.queue_glyph(self.font_id.0, glyph.clone());
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
                .filter_map(|g| glyph_cache.rect_for(self.font_id.0, g).ok().unwrap_or(None))
                .map(|(uv_rect, screen_rect)| {
                    (util::map_rect_i32(screen_rect), util::map_rect_f32(uv_rect) * tex_dim)
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

#[derive(Debug, Clone)]
pub enum TextStyleField {
    Text(Value<String>),
    FontId(Value<FontId>),
    FontSize(Value<Scalar>),
    TextColor(Value<Color>),
    BackgroundColor(Value<Color>),
    Wrap(Value<Wrap>),
    Align(Value<Align>),
}
impl StyleField<TextDrawable> for TextStyleField {
    fn apply(&self, state: &mut TextDrawable, props: &PropSet) {
        match *self {
            TextStyleField::Text(ref val) => state.text = val.from_props(props),
            TextStyleField::FontId(ref val) => state.font_id = val.from_props(props),
            TextStyleField::FontSize(ref val) => state.font_size = val.from_props(props),
            TextStyleField::TextColor(ref val) => state.text_color = val.from_props(props),
            TextStyleField::BackgroundColor(ref val) => {
                state.background_color = val.from_props(props)
            }
            TextStyleField::Wrap(ref val) => state.wrap = val.from_props(props),
            TextStyleField::Align(ref val) => state.align = val.from_props(props),
        }
    }
}
