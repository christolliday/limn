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

pub trait EventListener {
    fn handle_event(&mut self, widget: &mut Any, event: &Event);
    fn matches(&self, event: &Event) -> bool {
        false
    }
}

pub struct Widget {
    pub draw_fn: fn(&Any, Rectangle, &mut Resources, Context, &mut G2d),
    pub mouse_over_fn: fn(Point, Rectangle) -> bool,
    pub drawable: Box<Any>,
    pub layout: WidgetLayout,
    pub listeners: Vec<Box<EventListener>>,
    pub registered: Vec<EventId>,
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
            listeners: Vec::new(),
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
    pub fn handle_event(&mut self, solver: &mut Solver, event: &Event) {
        match event {
            &Event::Input(Input::Move(Motion::MouseCursor(x, y))) => {
                let pos = Point { x: x, y: y };
                let is_mouse_over = self.is_mouse_over(solver, pos);
                for listener in &mut self.listeners {
                    let matches = listener.matches(event);
                    if is_mouse_over && matches {
                        listener.handle_event(self.drawable.as_mut(), event);
                    }
                }
            }
            _ => {}
        }
    }
}
