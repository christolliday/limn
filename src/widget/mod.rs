pub mod layout;
pub mod primitives;
pub mod text;
pub mod image;

use backend::gfx::G2d;
use graphics::Context;

use input::Event;
use input::EventId;
use super::util::*;

use super::ui::Resources;
use self::layout::WidgetLayout;

use cassowary::Solver;

use std::any::Any;

pub struct Widget {
    pub draw_fn: fn(&Any, Rectangle, &mut Resources, Context, &mut G2d),
    pub mouse_over_fn: fn(Point, Rectangle) -> bool,
    pub drawable: Box<Any>,
    pub layout: WidgetLayout,
    pub registered: Vec<(EventId, fn(&mut Any, &Event))>,
}

use input::{Input, Motion};
impl Widget {
    pub fn new(draw_fn: fn(&Any, Rectangle, &mut Resources, Context, &mut G2d),
               drawable: Box<Any>)
               -> Self {
        Widget {
            draw_fn: draw_fn,
            mouse_over_fn: point_inside_rect,
            drawable: drawable,
            layout: WidgetLayout::new(),
            registered: Vec::new(),
        }
    }
    pub fn print(&self, solver: &mut Solver) {
        println!("{:?}", self.layout.bounds(solver));
    }
    pub fn draw(&self, resources: &mut Resources, solver: &mut Solver, c: Context, g: &mut G2d) {
        let bounds = self.layout.bounds(solver);
        (self.draw_fn)(self.drawable.as_ref(), bounds, resources, c, g);
    }
    pub fn is_mouse_over(&self, solver: &mut Solver, mouse: Point) -> bool {
        let bounds = self.layout.bounds(solver);
        (self.mouse_over_fn)(mouse, bounds)
    }
    pub fn trigger_event(&mut self, id: EventId, event: &Event) {
        let registered = self.registered.iter().find(|event_id| (*event_id).0 == id).unwrap();
        (registered.1)(self.drawable.as_mut(), event);
    }
}
