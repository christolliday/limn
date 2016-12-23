
use graphics;
use super::super::util::*;
use super::super::ui::Resources;
use backend::gfx::G2d;
use graphics::Context;
use graphics::types::Color;
use super::WidgetDrawable;

pub struct RectDrawable {
    pub background: Color,
}
impl WidgetDrawable for RectDrawable {
    fn draw(&self,
            bounds: Rectangle,
            resources: &mut Resources,
            context: Context,
            graphics: &mut G2d) {
        graphics::Rectangle::new(self.background)
            .draw(bounds, &context.draw_state, context.transform, graphics);
    }
}

pub struct EllipseDrawable {
    pub background: Color,
}
impl WidgetDrawable for EllipseDrawable {
    fn draw(&self,
            bounds: Rectangle,
            resources: &mut Resources,
            context: Context,
            graphics: &mut G2d) {
        graphics::Ellipse::new(self.background)
            .draw(bounds, &context.draw_state, context.transform, graphics);
    }
    fn is_mouse_over(&self, mouse: Point, bounds: Rectangle) -> bool {
        let radius = Dimensions {
            width: bounds.width / 2.0,
            height: bounds.height / 2.0,
        };
        let center = Point {
            x: bounds.left + radius.width,
            y: bounds.top + radius.height,
        };
        point_inside_ellipse(mouse, center, radius)
    }
}