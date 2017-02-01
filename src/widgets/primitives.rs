use std::collections::BTreeSet;

use graphics;
use graphics::types::Color;
use widget::{Drawable, WidgetStyle, StyleArgs, DrawArgs, PropSet};
use widget::style::{StyleSheet, DrawableStyle};

pub fn rect_drawable(style: RectStyle) -> Drawable {
    let mut drawable = Drawable::new(Box::new(RectDrawState::new(&style)), draw_rect);
    drawable.style = Some(WidgetStyle::new(Box::new(style), apply_rect_style));
    drawable
}

pub struct RectDrawState {
    pub background: Color,
}
impl RectDrawState {
    pub fn new(style: &RectStyle) -> Self {
        RectDrawState { background: style.background.default }
    }
}
pub fn draw_rect(draw_args: DrawArgs) {
    let DrawArgs { state, bounds, context, graphics, .. } = draw_args;
    let state: &RectDrawState = state.downcast_ref().unwrap();
    graphics::Rectangle::new(state.background)
        .draw(bounds, &context.draw_state, context.transform, graphics);
}

pub fn apply_rect_style(args: StyleArgs) {
    let state: &mut RectDrawState = args.state.downcast_mut().unwrap();
    let style: &RectStyle = args.style.downcast_ref().unwrap();
    style.apply(state, args.props);
}
#[derive(Clone)]
pub struct RectStyle {
    pub background: StyleSheet<Color>,
}
impl DrawableStyle<RectDrawState> for RectStyle {
    fn apply(&self, drawable: &mut RectDrawState, props: &PropSet) {
        drawable.background = self.background.apply(props).clone();
    }
}

pub fn ellipse_drawable(background: Color, border: Option<graphics::ellipse::Border>) -> Drawable {
    let mut drawable = Drawable::new(Box::new(EllipseDrawState { background: background, border: border }), draw_ellipse);
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
