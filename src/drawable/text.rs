use webrender_api::{LayoutPoint, GlyphInstance, FontKey};
use app_units;
use rusttype::{Scale, GlyphId, VMetrics};

use render::RenderBuilder;
use text_layout::{self, Wrap, Align};
use resources::resources;
use util::{self, Size, Rect, RectExt, Vector};
use widget::drawable::Drawable;
use widget::property::PropSet;
use widget::style::{Value, Styleable};
use color::*;

const DEBUG_LINE_BOUNDS: bool = false;

pub struct TextDrawable {
    pub text: String,
    pub font: String,
    pub font_size: f32,
    pub text_color: Color,
    pub background_color: Color,
    pub wrap: Wrap,
    pub align: Align,
}
impl Default for TextDrawable {
    fn default() -> Self {
        TextDrawable {
            text: "".to_owned(),
            font: "NotoSans/NotoSans-Regular".to_owned(),
            font_size: 30.0,
            text_color: BLACK,
            background_color: TRANSPARENT,
            wrap: Wrap::Whitespace,
            align: Align::Start,
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
        let line_height = self.line_height();
        let mut resources = resources();
        let font = resources.get_font(&self.font);
        text_layout::get_text_size(
            &self.text,
            &font.info,
            self.font_size,
            line_height,
            self.wrap)
    }
    pub fn min_height(&self) -> f32 {
        self.line_height()
    }
    pub fn line_height(&self) -> f32 {
        self.font_size + self.v_metrics().line_gap
    }
    pub fn text_fits(&self, text: &str, bounds: Rect) -> bool {
        let line_height = self.line_height();
        let mut resources = resources();
        let font = resources.get_font(&self.font);
        let height = text_layout::get_text_height(
            text,
            &font.info,
            self.font_size,
            line_height,
            self.wrap,
            bounds.width());
        height < bounds.height()
    }
    fn get_line_rects(&self, bounds: Rect) -> Vec<Rect> {
        let line_height = self.line_height();
        let mut resources = resources();
        let font = resources.get_font(&self.font);
        text_layout::get_line_rects(
            &self.text,
            bounds,
            &font.info,
            self.font_size,
            line_height,
            self.wrap,
            self.align)
    }
    fn position_glyphs(&self, bounds: Rect) -> Vec<GlyphInstance> {
        let line_height = self.line_height();
        let descent = self.v_metrics().descent;
        let mut resources = resources();
        let font = resources.get_font(&self.font);
        let positions = text_layout::get_positioned_glyphs(
            &self.text,
            bounds,
            &font.info,
            self.font_size,
            line_height,
            self.wrap,
            self.align).iter().map(|glyph| {
                let position = glyph.position();
                GlyphInstance {
                    index: glyph.id().0,
                    point: LayoutPoint::new(position.x, position.y + descent),
                }
            }).collect();
        positions
    }
    fn font_key(&self) -> FontKey {
        resources().get_font(&self.font).key
    }
    fn v_metrics(&self) -> VMetrics {
        let mut resources = resources();
        let font = resources.get_font(&self.font);
        font.info.v_metrics(Scale::uniform(self.font_size))
    }
}

impl Drawable for TextDrawable {
    fn draw(&mut self, bounds: Rect, _: Rect, renderer: &mut RenderBuilder) {
        let glyphs = self.position_glyphs(bounds);
        if DEBUG_LINE_BOUNDS {
            let line_rects = self.get_line_rects(bounds);
            let v_metrics = self.v_metrics();
            let mut resources = resources();
            let font = resources.get_font(&self.font);
            for mut rect in line_rects {
                util::draw_rect_outline(rect, CYAN, renderer);
                rect.origin.y = rect.bottom() + v_metrics.descent;
                rect.size.height = 1.0;
                util::draw_rect_outline(rect, RED, renderer);
            }
            let scale = Scale::uniform(self.font_size);
            for glyph in &glyphs {
                let scaled_glyph = font.info.glyph(GlyphId(glyph.index)).unwrap().scaled(scale);
                if let Some(rect) = scaled_glyph.exact_bounding_box() {
                    let origin = glyph.point.to_vector().to_untyped() + Vector::new(0.0, -1.0);
                    let rect = Rect::from_rusttype(rect).translate(&origin);
                    util::draw_rect_outline(rect, BLUE, renderer);
                }
            }
        }
        let size = app_units::Au::from_f32_px(text_layout::px_to_pt(self.font_size));
        let key = self.font_key();
        renderer.builder.push_text(
            bounds.typed(),
            None,
            &glyphs,
            key,
            self.text_color.into(),
            size,
            None
        );
    }
}

#[derive(Debug, Clone)]
pub enum TextStyleable {
    Text(Value<String>),
    Font(Value<String>),
    FontSize(Value<f32>),
    TextColor(Value<Color>),
    BackgroundColor(Value<Color>),
    Wrap(Value<Wrap>),
    Align(Value<Align>),
}

impl Styleable<TextDrawable> for TextStyleable {
    fn apply(&self, state: &mut TextDrawable, props: &PropSet) {
        match *self {
            TextStyleable::Text(ref val) => state.text = val.get(props),
            TextStyleable::Font(ref val) => state.font = val.get(props),
            TextStyleable::FontSize(ref val) => state.font_size = val.get(props),
            TextStyleable::TextColor(ref val) => state.text_color = val.get(props),
            TextStyleable::BackgroundColor(ref val) => state.background_color = val.get(props),
            TextStyleable::Wrap(ref val) => state.wrap = val.get(props),
            TextStyleable::Align(ref val) => state.align = val.get(props),
        }
    }
}
