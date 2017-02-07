use std::collections::BTreeSet;
use std::any::Any;
use std::ops::Index;

use graphics;
use graphics::types::Color;
use widget::{Drawable, WidgetStyle, StyleArgs, DrawArgs, PropSet};
use widget::style::Value;

pub fn rect_drawable(style: RectStyle) -> Drawable {
    let draw_state = RectDrawState { background: style.background.default() };
    let mut drawable = Drawable::new(draw_state, draw_rect);
    drawable.style = Some(WidgetStyle::new(style, apply_rect_style));
    drawable
}

pub struct RectDrawState {
    pub background: Color,
}
pub enum RectField {
    Background
}

impl<RectField> Index<RectField> for RectDrawState {
    type Output = Any;
    fn index(&self, field: RectField) -> &Any {
        match field {
            Background => &self.background
        }
    }
}

pub fn draw_rect(args: DrawArgs) {
    let DrawArgs { state, bounds, context, graphics, .. } = args;
    let state: &RectDrawState = state.downcast_ref().unwrap();
    graphics::Rectangle::new(state.background)
        .draw(bounds, &context.draw_state, context.transform, graphics);
}

pub fn apply_rect_style(args: StyleArgs) {
    let state: &mut RectDrawState = args.state.downcast_mut().unwrap();
    let style: &RectStyle = args.style.downcast_ref().unwrap();
    state.background = style.background.from_props(&args.props);
}
#[derive(Clone)]
pub struct RectStyle {
    pub background: Value<Color>,
}

pub fn ellipse_drawable(background: Color, border: Option<graphics::ellipse::Border>) -> Drawable {
    let draw_state = EllipseDrawState { background: background, border: border };
    let mut drawable = Drawable::new(draw_state, draw_ellipse);
    drawable
}
pub struct EllipseDrawState {
    pub background: Color,
    pub border: Option<graphics::ellipse::Border>,
}
pub fn draw_ellipse(draw_args: DrawArgs) {
    let DrawArgs { state, bounds, context, graphics, .. } = draw_args;
    let state: &EllipseDrawState = state.downcast_ref().unwrap();

    graphics::Ellipse::new(state.background)
        .maybe_border(state.border)
        .draw(bounds, &context.draw_state, context.transform, graphics);
}
