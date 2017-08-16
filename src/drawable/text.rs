use graphics::types::Color;

use render::RenderBuilder;
use text_layout::{self, Wrap, Align};
use resources::{FontId, resources};
use util::{self, Point, Size, SizeExt, Scalar, Rect, RectExt};
use widget::drawable::Drawable;
use widget::property::PropSet;
use widget::style::{Value, Styleable};
use color::*;

const DEBUG_LINE_BOUNDS: bool = false;

pub struct TextDrawable {
    pub text: String,
    pub font_id: FontId,
    pub font_size: Scalar,
    pub text_color: Color,
    pub background_color: Color,
    pub wrap: Wrap,
    pub align: Align,
    pub vertical_align: Align,
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
            vertical_align: Align::Middle,
        }
    }
}

impl TextDrawable {
    pub fn new(text: &str) -> Self {
        let mut drawable = TextDrawable::default();
        drawable.text = text.to_owned();
        drawable
    }
    pub fn measure(&self) -> Size {
        let res = resources();
        let font = res.fonts.get(self.font_id).unwrap();

        let dims = text_layout::get_text_dimensions(
            &self.text,
            font,
            self.font_size,
            self.font_size * 1.25,
            self.wrap);
        Size::from_text_layout(dims)
    }
    pub fn min_height(&self) -> Scalar {
        self.font_size * 1.25
    }
    pub fn text_fits(&self, text: &str, bounds: Rect) -> bool {
        let res = resources();
        let font = res.fonts.get(self.font_id).unwrap();
        let height =
            text_layout::get_text_height(text,
                                         font,
                                         self.font_size,
                                         self.font_size * 1.25,
                                         self.wrap,
                                         bounds.width());
        height < bounds.height()
    }
}

impl Drawable for TextDrawable {
    fn draw(&mut self, bounds: Rect, _: Rect, renderer: &mut RenderBuilder) {
        /*
        graphics::Rectangle::new(self.background_color)
                .draw(bounds.to_slice(), &context.draw_state, context.transform, graphics);

            let &mut GlyphCache { texture: ref mut text_texture_cache,
                                cache: ref mut glyph_cache,
                                ref mut vertex_data } = glyph_cache;

            let res = resources();
            let font = res.fonts.get(self.font_id).unwrap();

            let line_height = self.font_size * 1.25;
            if DEBUG_LINE_BOUNDS {
                let line_rects = &text_layout::get_line_rects(&self.text,
                                                              bounds.to_text_layout(),
                                                              font,
                                                              self.font_size,
                                                              line_height,
                                                              self.wrap,
                                                              self.align,
                                                              self.vertical_align);
                for line_rect in line_rects {
                    let rect = Rect::from_text_layout(*line_rect);
                    util::draw_rect_outline(rect, CYAN, context, graphics);
                }
            }
            let positioned_glyphs = &text_layout::get_positioned_glyphs(&self.text,
                                                                        bounds.to_text_layout(),
                                                                        font,
                                                                        self.font_size,
                                                                        line_height,
                                                                        self.wrap,
                                                                        self.align,
                                                                        self.vertical_align);

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
                Size::new(tex_w as f64, tex_h as f64)
            };

            let scale_rect = |rect: Rect, size: Size| -> Rect {
                Rect::new(
                    Point::new(rect.left() * size.width, rect.top() * size.height),
                    Size::new(rect.width() * size.width, rect.height() * size.height),
                )
            };
            let rectangles = positioned_glyphs.into_iter()
                .filter_map(|g| glyph_cache.rect_for(self.font_id.0, g).ok().unwrap_or(None))
                .map(|(uv_rect, screen_rect)| {
                    let screen_rect = Rect::from_rusttype(screen_rect).to_slice();
                    let uv_rect = scale_rect(Rect::from_rusttype(uv_rect), tex_dim).to_slice();
                    (screen_rect, uv_rect)
                });
            // Contains each glyph's screen and texture positions.
            let mut glyph_rectangles = Vec::new();
            glyph_rectangles.extend(rectangles);
            graphics::image::draw_many(&glyph_rectangles,
                                       self.text_color,
                                       text_texture_cache,
                                       &context.draw_state,
                                       context.transform,
                                       graphics);
                                       */
    }
}

#[derive(Debug, Clone)]
pub enum TextStyleable {
    Text(Value<String>),
    FontId(Value<FontId>),
    FontSize(Value<Scalar>),
    TextColor(Value<Color>),
    BackgroundColor(Value<Color>),
    Wrap(Value<Wrap>),
    Align(Value<Align>),
    VertAlign(Value<Align>),
}

impl Styleable<TextDrawable> for TextStyleable {
    fn apply(&self, state: &mut TextDrawable, props: &PropSet) {
        match *self {
            TextStyleable::Text(ref val) => state.text = val.get(props),
            TextStyleable::FontId(ref val) => state.font_id = val.get(props),
            TextStyleable::FontSize(ref val) => state.font_size = val.get(props),
            TextStyleable::TextColor(ref val) => state.text_color = val.get(props),
            TextStyleable::BackgroundColor(ref val) => state.background_color = val.get(props),
            TextStyleable::Wrap(ref val) => state.wrap = val.get(props),
            TextStyleable::Align(ref val) => state.align = val.get(props),
            TextStyleable::VertAlign(ref val) => state.vertical_align = val.get(props),
        }
    }
}
