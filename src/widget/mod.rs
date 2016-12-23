pub mod layout;
pub mod primitives;
pub mod text;
pub mod image;

use backend::gfx::G2d;
use graphics;
use graphics::Context;
use graphics::types::Color;

use input::Event;
use super::util::*;

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
pub struct EmptyDrawable {}
impl WidgetDrawable for EmptyDrawable {
    fn draw(&self,
            bounds: Rectangle,
            resources: &mut Resources,
            context: Context,
            graphics: &mut G2d) {}
}

pub struct Widget {
    pub drawable: Box<WidgetDrawable>,
    pub layout: WidgetLayout,
    pub listeners: Vec<Box<EventListener>>,
}

impl Widget {
    pub fn new(drawable: Box<WidgetDrawable>) -> Self {
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
        self.drawable.draw(self.layout.bounds(solver), resources, c, g);
    }
    pub fn is_mouse_over(&self, solver: &mut Solver, mouse: Point) -> bool {
        let bounds = self.layout.bounds(solver);
        self.drawable.is_mouse_over(mouse, bounds)
    }
}
