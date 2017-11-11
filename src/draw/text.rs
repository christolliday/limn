use webrender::api::{LayoutPoint, GlyphInstance, PrimitiveInfo, FontInstanceKey};
use rusttype::{Scale, GlyphId, VMetrics};

use render::RenderBuilder;
use text_layout::{self, Wrap, Align};
use resources::resources;
use geometry::{Size, Rect, RectExt, Vector};
use render;
use widget::draw::Draw;
use widget::property::PropSet;
use widget::style::{self, Value, PropSelector};
use color::*;
use style::*;

const DEBUG_LINE_BOUNDS: bool = false;

#[derive(Debug, Default, Clone)]
pub struct TextState {
    pub text: String,
    pub font: String,
    pub font_size: f32,
    pub text_color: Color,
    pub background_color: Color,
    pub wrap: Wrap,
    pub align: Align,
}

impl Component for TextState {
    fn name() -> String {
        String::from("text")
    }
}

impl TextState {
    pub fn measure(&self) -> Size {
        let line_height = self.line_height();
        let mut resources = resources();
        let font = resources.get_font(&self.font);
        Size::from_untyped(&text_layout::get_text_size(
            &self.text,
            &font.info,
            self.font_size,
            line_height,
            self.wrap))
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
        height <= bounds.height()
    }
    fn get_line_rects(&self, bounds: Rect) -> Vec<Rect> {
        let line_height = self.line_height();
        let mut resources = resources();
        let font = resources.get_font(&self.font);
        text_layout::get_line_rects(
            &self.text,
            bounds.to_untyped(),
            &font.info,
            self.font_size,
            line_height,
            self.wrap,
            self.align).iter().map(|rect| Rect::from_untyped(rect)).collect()
    }
    fn position_glyphs(&self, bounds: Rect) -> Vec<GlyphInstance> {
        let line_height = self.line_height();
        let descent = self.v_metrics().descent;
        let mut resources = resources();
        let font = resources.get_font(&self.font);
        text_layout::get_positioned_glyphs(
            &self.text,
            bounds.to_untyped(),
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
        let font = resources.get_font(&self.font);
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
            let font = resources.get_font(&self.font);
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
                    let origin = glyph.point.to_vector() + Vector::new(0.0, -1.0);
                    let rect = Rect::from_rusttype(rect).translate(&origin);
                    render::draw_rect_outline(rect, BLUE, renderer);
                }
            }
        }
        let key = self.font_instance_key();
        let info = PrimitiveInfo::new(bounds);
        renderer.builder.push_text(
            &info,
            &glyphs,
            key,
            self.text_color.into(),
            None,
        );
    }
}

#[derive(Default, Clone)]
pub struct TextComponentStyle {
    pub text: Option<Value<String>>,
    pub font: Option<Value<String>>,
    pub font_size: Option<Value<f32>>,
    pub text_color: Option<Value<Color>>,
    pub background_color: Option<Value<Color>>,
    pub wrap: Option<Value<Wrap>>,
    pub align: Option<Value<Align>>,
}

impl TextComponentStyle {
    pub fn new(text: &str) -> Self {
        TextComponentStyle {
            text: Some(Value::from(text.to_owned())),
            ..TextComponentStyle::default()
        }
    }
}

impl ComponentStyle for TextComponentStyle {
    type Component = TextComponent;
    fn merge(&self, other: &Self) -> Self {
        TextComponentStyle {
            text: self.text.as_ref().or(other.text.as_ref()).cloned(),
            font: self.font.as_ref().or(other.font.as_ref()).cloned(),
            font_size: self.font_size.as_ref().or(other.font_size.as_ref()).cloned(),
            text_color: self.text_color.as_ref().or(other.text_color.as_ref()).cloned(),
            background_color: self.background_color.as_ref().or(other.background_color.as_ref()).cloned(),
            wrap: self.wrap.as_ref().or(other.wrap.as_ref()).cloned(),
            align: self.align.as_ref().or(other.align.as_ref()).cloned(),
        }
    }
    fn component(self) -> Self::Component {
        TextComponent {
            text: self.text.unwrap_or(Value::from("".to_owned())),
            font: self.font.unwrap_or(Value::from("NotoSans/NotoSans-Regular".to_owned())),
            font_size: self.font_size.unwrap_or(Value::from(24.0)),
            text_color: self.text_color.unwrap_or(Value::from(BLACK)),
            background_color: self.background_color.unwrap_or(Value::from(TRANSPARENT)),
            wrap: self.wrap.unwrap_or(Value::from(Wrap::Whitespace)),
            align: self.align.unwrap_or(Value::from(Align::Start)),
        }
    }
}

#[derive(Clone)]
pub struct TextComponent {
    pub text: Value<String>,
    pub font: Value<String>,
    pub font_size: Value<f32>,
    pub text_color: Value<Color>,
    pub background_color: Value<Color>,
    pub wrap: Value<Wrap>,
    pub align: Value<Align>,
}

impl Component for TextComponent {
    fn name() -> String {
        "text".to_owned()
    }
}

impl PropSelector<TextState> for TextComponent {
    fn apply(&self, state: &mut TextState, props: &PropSet) -> bool {
        let res = style::update(&mut state.text, self.text.get(props)) |
        style::update(&mut state.font, self.font.get(props)) |
        style::update(&mut state.font_size, self.font_size.get(props)) |
        style::update(&mut state.text_color, self.text_color.get(props)) |
        style::update(&mut state.background_color, self.background_color.get(props)) |
        style::update(&mut state.wrap, self.wrap.get(props)) |
        style::update(&mut state.align, self.align.get(props));

        res
    }
}
