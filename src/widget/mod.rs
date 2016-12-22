pub mod layout;
pub mod text;

use backend::gfx::G2d;
use graphics;
use graphics::Context;
use graphics::types::Color;

use input::Event;
use super::util::*;

//use super::text;
use backend::glyph;
use super::ui::Resources;
use self::layout::WidgetLayout;

use cassowary::{Solver, Variable, Constraint};
use cassowary::WeightedRelation::*;
use cassowary::strength::*;

pub trait EventListener {
    fn handle_event(&self, event: &Event);
    fn matches(&self, event: &Event) -> bool {
        false
    }
}

pub trait WidgetDrawable {
    fn draw(&self,
            bounds: Rectangle,
            resources: &mut Resources,
            context: Context,
            graphics: &mut G2d);
    fn is_mouse_over(&self, mouse: Point, bounds: Rectangle) -> bool {
        point_inside_rect(mouse, bounds)
    }
}

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

pub struct Widget {
    pub drawable: Option<Box<WidgetDrawable>>,
    pub layout: WidgetLayout,
    pub listeners: Vec<Box<EventListener>>,
}

impl Widget {
    pub fn new(drawable: Option<Box<WidgetDrawable>>) -> Self {
        Widget {
            drawable: drawable,
            layout: WidgetLayout::new(),
            listeners: Vec::new(),
        }
    }
    pub fn print(&self, solver: &mut Solver) {
        println!("{:?}", self.layout.bounds(solver));
    }
    pub fn draw(&self,
                resources: &mut Resources,
                solver: &mut Solver,
                c: Context,
                g: &mut G2d) {
        if let Some(ref drawable) = self.drawable {
            drawable.draw(self.layout.bounds(solver), resources, c, g);
        }
    }
    pub fn is_mouse_over(&self, solver: &mut Solver, mouse: Point) -> bool {
        let bounds = self.layout.bounds(solver);
        if let Some(ref drawable) = self.drawable {
            drawable.is_mouse_over(mouse, bounds)
        } else {
            point_inside_rect(mouse, bounds)
        }
    }
}
