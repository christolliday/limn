
use graphics;
use super::super::util::*;
use super::super::ui::Resources;
use backend::gfx::G2d;
use graphics::Context;
use graphics::types::Color;
use std::any::Any;

pub struct EmptyDrawable {}
pub fn draw_nothing(state: &Any,
            bounds: Rectangle,
            resources: &mut Resources,
            context: Context,
            graphics: &mut G2d) {}

pub struct RectDrawable {
    pub background: Color,
}
pub fn draw_rect(state: &Any,
            bounds: Rectangle,
            resources: &mut Resources,
            context: Context,
            graphics: &mut G2d)
{
    let state: &RectDrawable = state.downcast_ref().unwrap();
    graphics::Rectangle::new(state.background)
            .draw(bounds, &context.draw_state, context.transform, graphics);
}

pub struct EllipseDrawable {
    pub background: Color,
}
pub fn draw_ellipse(state: &Any,
            bounds: Rectangle,
            resources: &mut Resources,
            context: Context,
            graphics: &mut G2d)
{
    let state: &EllipseDrawable = state.downcast_ref().unwrap();

        graphics::Ellipse::new(state.background)
            .draw(bounds, &context.draw_state, context.transform, graphics);
}