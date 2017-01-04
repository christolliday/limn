use graphics;
use super::super::util::*;
use super::super::ui::Resources;
use backend::gfx::G2d;
use graphics::Context;
use graphics::types::Color;
use std::any::Any;
use widget::DrawArgs;

pub struct RectDrawable {
    pub background: Color,
}
pub fn draw_rect(draw_args: DrawArgs) {
    let DrawArgs { state, bounds, context, graphics, .. } = draw_args;
    let state: &RectDrawable = state.downcast_ref().unwrap();
    graphics::Rectangle::new(state.background)
        .draw(bounds, &context.draw_state, context.transform, graphics);
}

pub struct EllipseDrawable {
    pub background: Color,
}
pub fn draw_ellipse(draw_args: DrawArgs) {
    let DrawArgs { state, bounds, context, graphics, .. } = draw_args;
    let state: &EllipseDrawable = state.downcast_ref().unwrap();

    graphics::Ellipse::new(state.background)
        .draw(bounds, &context.draw_state, context.transform, graphics);
}
