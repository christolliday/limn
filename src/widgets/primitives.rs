use std::collections::BTreeSet;

use graphics;
use graphics::types::Color;
use widget::{StyleArgs, DrawArgs, WidgetProperty};
use widget::style::{StyleSheet, DrawableStyle};

pub struct RectDrawable {
    pub background: Color,
}
impl RectDrawable {
    pub fn new(style: &RectStyle) -> Self {
        RectDrawable { background: style.background.default }
    }
}
pub fn draw_rect(draw_args: DrawArgs) {
    let DrawArgs { state, bounds, context, graphics, .. } = draw_args;
    let state: &RectDrawable = state.downcast_ref().unwrap();
    graphics::Rectangle::new(state.background)
        .draw(bounds, &context.draw_state, context.transform, graphics);
}

pub struct EllipseDrawable {
    pub background: Color,
    pub border: Option<graphics::ellipse::Border>,
}
pub fn draw_ellipse(draw_args: DrawArgs) {
    let DrawArgs { state, bounds, context, graphics, .. } = draw_args;
    let state: &EllipseDrawable = state.downcast_ref().unwrap();

    graphics::Ellipse::new(state.background)
        .maybe_border(state.border)
        .draw(bounds, &context.draw_state, context.transform, graphics);
}

pub fn apply_rect_style(args: StyleArgs) {
    let state: &mut RectDrawable = args.state.downcast_mut().unwrap();
    let style: &RectStyle = args.style.downcast_ref().unwrap();
    style.apply(state, args.props);
}
pub struct RectStyle {
    pub background: StyleSheet<Color>,
}
impl DrawableStyle<RectDrawable> for RectStyle {
    fn apply(&self, drawable: &mut RectDrawable, props: &BTreeSet<WidgetProperty>) {
        drawable.background = self.background.apply(props).clone();
    }
}