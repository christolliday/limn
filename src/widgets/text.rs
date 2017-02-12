use std::collections::HashSet;
use std::any::Any;
use std::ops::{Index, IndexMut};

use graphics;
use graphics::types::Color;

use backend::glyph::{self, GlyphCache};
use backend::gfx::ImageSize;

use text::{self, Wrap};
use resources::{Id, FontId, resources};
use util::{self, Dimensions, Align, Scalar};
use color::*;
use widget::{Drawable, WidgetStyle, StyleArgs, DrawArgs, Property, PropSet};
use widget::style::Value;
use theme::STYLE_TEXT;

pub fn text_drawable(style: TextStyle) -> Drawable {
    let draw_state = TextDrawState::new(&style);
    let mut drawable = Drawable::new(draw_state, draw_text);
    drawable.style = Some(WidgetStyle::new(style, apply_text_style));
    drawable
}

pub struct TextDrawState {
    pub text: String,
    pub font_id: FontId,
    pub font_size: Scalar,
    pub text_color: Color,
    pub background_color: Color,
    pub wrap: Wrap,
    pub align: Align,
}

pub fn apply_text_style(args: StyleArgs) {
    let state: &mut TextDrawState = args.state.downcast_mut().unwrap();
    let style: &TextStyle = args.style.downcast_ref().unwrap();
    let props = args.props;
    state.text = style.text.from_props(props);
    state.font_id = style.font_id.from_props(props);
    state.font_size = style.font_size.from_props(props);
    state.text_color = style.text_color.from_props(props);
    state.background_color = style.background_color.from_props(props);
    state.wrap = style.wrap.from_props(props);
    state.align = style.align.from_props(props);
}

#[derive(Clone)]
pub struct TextStyle {
    pub text: Value<String>,
    pub font_id: Value<FontId>,
    pub font_size: Value<Scalar>,
    pub text_color: Value<Color>,
    pub background_color: Value<Color>,
    pub wrap: Value<Wrap>,
    pub align: Value<Align>,
}

#[derive(Debug)]
pub enum TextStyleField {
    text(Value<String>),
    font_id(Value<FontId>),
    font_size(Value<Scalar>),
    text_color(Value<Color>),
    background_color(Value<Color>),
    wrap(Value<Wrap>),
    align(Value<Align>),
}

impl TextStyle {
    pub fn from(fields: Vec<TextStyleField>) -> Self {
        let mut style = STYLE_TEXT.clone();
        style.extend(fields);
        style
    }
    pub fn extend(&mut self, mut style: Vec<TextStyleField>) {
        for field in style.drain(..) {
            match field {
                TextStyleField::text(val) => self.text = val,
                TextStyleField::font_id(val) => self.font_id = val,
                TextStyleField::font_size(val) => self.font_size = val,
                TextStyleField::text_color(val) => self.text_color = val,
                TextStyleField::background_color(val) => self.background_color = val,
                TextStyleField::wrap(val) => self.wrap = val,
                TextStyleField::align(val) => self.align = val,
            }
        }
    }
}

pub fn measure(drawable: &Drawable) -> Dimensions {
    let draw_state: &TextDrawState = drawable.state();
    draw_state.measure()
}

impl TextDrawState {
    pub fn new(style: &TextStyle) -> Self {
        TextDrawState {
            text: style.text.default(),
            font_id: style.font_id.default(),
            font_size: style.font_size.default(),
            text_color: style.text_color.default(),
            background_color: style.background_color.default(),
            wrap: style.wrap.default(),
            align: style.align.default(),
        }
    }
    pub fn measure(&self) -> Dimensions {
        let res = resources();
        let font = res.fonts.get(self.font_id).unwrap();
        text::get_text_dimensions(&self.text,
                                  font,
                                  self.font_size,
                                  self.font_size * 1.25,
                                  self.wrap)
    }
}

pub fn draw_text(draw_args: DrawArgs) {

    let DrawArgs { state, bounds, glyph_cache, context, graphics, .. } = draw_args;
    let state: &TextDrawState = state.downcast_ref().unwrap();

    graphics::Rectangle::new(state.background_color)
        .draw(bounds, &context.draw_state, context.transform, graphics);

    let &mut GlyphCache { texture: ref mut text_texture_cache,
                          cache: ref mut glyph_cache,
                          ref mut vertex_data } = glyph_cache;

    let res = resources();
    let font = res.fonts.get(state.font_id).unwrap();

    let positioned_glyphs = &text::get_positioned_glyphs(&state.text,
                                                         bounds,
                                                         font,
                                                         state.font_size,
                                                         state.font_size * 1.25,
                                                         state.wrap,
                                                         state.align,
                                                         Align::Start);

    // Queue the glyphs to be cached.
    for glyph in positioned_glyphs.iter() {
        glyph_cache.queue_glyph(state.font_id.0, glyph.clone());
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
        .filter_map(|g| glyph_cache.rect_for(state.font_id.0, g).ok().unwrap_or(None))
        .map(|(uv_rect, screen_rect)| {
            (util::map_rect_i32(screen_rect), util::map_rect_f32(uv_rect) * tex_dim)
        });
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
