use webrender::api::{LayoutPoint, GlyphInstance, PrimitiveInfo, FontInstanceKey};
use rusttype::{Scale, GlyphId, VMetrics};

use render::RenderBuilder;
use text_layout::{self, Wrap, Align};
use resources::resources;
use geometry::{Size, Rect, RectExt, Vector};
use render;
use widget::draw::Draw;
use widget::property::PropSet;
use widget::style::{self, Value, Style};
use color::*;

const DEBUG_LINE_BOUNDS: bool = false;

pub struct TextState {
    pub text: String,
    pub font: String,
    pub font_size: f32,
    pub text_color: Color,
    pub background_color: Color,
    pub wrap: Wrap,
    pub align: Align,
}
impl Default for TextState {
    fn default() -> Self {
        TextState {
            text: "".to_owned(),
            font: "Sans".to_owned(),
            font_size: 24.0,
            text_color: BLACK,
            background_color: TRANSPARENT,
            wrap: Wrap::Whitespace,
            align: Align::Start,
        }
    }
}

impl TextState {
    pub fn new(text: &str) -> Self {
        let mut draw_state = TextState::default();
        draw_state.text = text.to_owned();
        draw_state
    }
    pub fn measure(&self) -> Size {
        let line_height = self.line_height();
        let mut resources = resources();
        let font = resources.get_font(&self.font).unwrap();
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
        let font = resources.get_font(&self.font).unwrap();
        let height = text_layout::get_text_height(
            text,
            &font.info,
            self.font_size,
            line_height,
            self.wrap,
            bounds.width());
        height <= bounds.height()
    }
    fn get_line_rects(&self, bounds: Rect) -> Vec<Rect> {
        let line_height = self.line_height();
        let mut resources = resources();
        let font = resources.get_font(&self.font).unwrap();
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
        let font = resources.get_font(&self.font).unwrap();
        text_layout::get_positioned_glyphs(
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
            }).collect()
    }
    fn font_instance_key(&self) -> FontInstanceKey {
        *resources().get_font_instance(&self.font, self.font_size)
    }
    fn v_metrics(&self) -> VMetrics {
        let mut resources = resources();
        let font = resources.get_font(&self.font).unwrap();
        font.info.v_metrics(Scale::uniform(self.font_size))
    }
}

impl Draw for TextState {
    fn draw(&mut self, bounds: Rect, _: Rect, renderer: &mut RenderBuilder) {
        let glyphs = self.position_glyphs(bounds);
        if DEBUG_LINE_BOUNDS {
            let line_rects = self.get_line_rects(bounds);
            let v_metrics = self.v_metrics();
            let mut resources = resources();
            let font = resources.get_font(&self.font).unwrap();
            for mut rect in line_rects {
                render::draw_rect_outline(rect, CYAN, renderer);
                rect.origin.y = rect.bottom() + v_metrics.descent;
                rect.size.height = 1.0;
                render::draw_rect_outline(rect, RED, renderer);
            }
            let scale = Scale::uniform(self.font_size);
            for glyph in &glyphs {
                let scaled_glyph = font.info.glyph(GlyphId(glyph.index)).unwrap().scaled(scale);
                if let Some(rect) = scaled_glyph.exact_bounding_box() {
                    let origin = glyph.point.to_vector().to_untyped() + Vector::new(0.0, -1.0);
                    let rect = Rect::from_rusttype(rect).translate(&origin);
                    render::draw_rect_outline(rect, BLUE, renderer);
                }
            }
        }
        let key = self.font_instance_key();
        let info = PrimitiveInfo::new(bounds.typed());
        renderer.builder.push_text(
            &info,
            &glyphs,
            key,
            self.text_color.into(),
            None,
        );
    }
}

#[derive(Debug, Clone)]
pub enum TextStyle {
    Text(Value<String>),
    Font(Value<String>),
    FontSize(Value<f32>),
    TextColor(Value<Color>),
    BackgroundColor(Value<Color>),
    Wrap(Value<Wrap>),
    Align(Value<Align>),
}

impl Style<TextState> for TextStyle {
    fn apply(&self, state: &mut TextState, props: &PropSet) -> bool {
        match *self {
            TextStyle::Text(ref val) => style::update(&mut state.text, val.get(props)),
            TextStyle::Font(ref val) => style::update(&mut state.font, val.get(props)),
            TextStyle::FontSize(ref val) => style::update(&mut state.font_size, val.get(props)),
            TextStyle::TextColor(ref val) => style::update(&mut state.text_color, val.get(props)),
            TextStyle::BackgroundColor(ref val) => style::update(&mut state.background_color, val.get(props)),
            TextStyle::Wrap(ref val) => style::update(&mut state.wrap, val.get(props)),
            TextStyle::Align(ref val) => style::update(&mut state.align, val.get(props)),
        }
    }
}
