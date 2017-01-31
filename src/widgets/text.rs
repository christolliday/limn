use std::collections::BTreeSet;

use graphics;
use graphics::types::Color;

use backend::glyph::{self, GlyphCache};
use backend::gfx::ImageSize;

use text::{self, Wrap};
use resources::{Id, resources};
use util::{self, Dimensions, Align, Scalar};
use color::*;
use widget::{StyleArgs, DrawArgs, WidgetProperty};
use widget::style::{DrawableStyle, StyleSheet};

pub struct TextDrawable {
    pub text: String,
    pub font_id: Id,
    pub font_size: Scalar,
    pub text_color: Color,
    pub background_color: Color,
}

pub fn apply_text_style(args: StyleArgs) {
    let state: &mut TextDrawable = args.state.downcast_mut().unwrap();
    let style: &TextStyle = args.style.downcast_ref().unwrap();
    style.apply(state, args.props);
}

pub struct TextStyle {
    font_id: StyleSheet<Id>,
    font_size: StyleSheet<Scalar>,
    text_color: StyleSheet<Color>,
    background_color: StyleSheet<Color>,
}
impl DrawableStyle<TextDrawable> for TextStyle {
    fn apply(&self, drawable: &mut TextDrawable, props: &BTreeSet<WidgetProperty>) {
        drawable.font_id = self.font_id.apply(props).clone();
        drawable.font_size = self.font_size.apply(props).clone();
        drawable.text_color = self.text_color.apply(props).clone();
        drawable.background_color = self.background_color.apply(props).clone();
    }
}

impl TextDrawable {
    pub fn new_default(text: String, font_id: Id) -> Self {
        TextDrawable {
            text: text,
            font_id: font_id,
            font_size: 24.0,
            text_color: BLACK,
            background_color: TRANSPARENT,
        }
    }
    pub fn new(text: String, font_id: Id, font_size: Scalar, text_color: Color, background_color: Color) -> Self {
        TextDrawable {
            text: text,
            font_id: font_id,
            font_size: font_size,
            text_color: text_color,
            background_color: background_color,
        }
    }
    pub fn measure_dims_no_wrap(&self) -> Dimensions {
        let res = resources();
        let font = res.fonts.get(self.font_id).unwrap();
        text::get_text_dimensions(&self.text,
                                  font,
                                  self.font_size,
                                  self.font_size * 1.25,
                                  Align::Start,
                                  Align::Start)
    }
    pub fn measure_height_wrapped(&self, width: Scalar) -> Scalar {
        let res = resources();
        let font = res.fonts.get(self.font_id).unwrap();
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

pub fn draw_text(draw_args: DrawArgs) {

    let DrawArgs { state, bounds, glyph_cache, context, graphics, .. } = draw_args;
    let state: &TextDrawable = state.downcast_ref().unwrap();

    graphics::Rectangle::new(state.background_color)
        .draw(bounds, &context.draw_state, context.transform, graphics);

    let &mut GlyphCache { texture: ref mut text_texture_cache,
                          cache: ref mut glyph_cache,
                          ref mut vertex_data } = glyph_cache;

    let res = resources();
    let font = res.fonts.get(state.font_id).unwrap();
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
        .map(|(uv_rect, screen_rect)| (util::map_rect_i32(screen_rect), util::map_rect_f32(uv_rect) * tex_dim));
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
