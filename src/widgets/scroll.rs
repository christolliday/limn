use std::any::Any;

use glutin;
use cassowary::strength::*;

use event::{self, EventId, EventAddress, WIDGET_REDRAW};
use widget::{EventArgs, EventHandler};
use util::{Point, Rectangle};

pub const SCROLL_SCROLLED: EventId = EventId("limn/scroll_scrolled");

pub struct ScrollHandler {}
impl EventHandler for ScrollHandler {
    fn event_id(&self) -> EventId {
        event::WIDGET_SCROLL
    }
    fn handle_event(&mut self, args: EventArgs) {
        let EventArgs { data, widget_id, layout, event_queue, solver, .. } = args;
        let event = data.downcast_ref::<glutin::Event>().unwrap();
        let widget_bounds = layout.bounds(solver);
        event_queue.push(EventAddress::Child(widget_id),
                         SCROLL_SCROLLED,
                         Box::new((event.clone(), widget_bounds)));
    }
}

pub struct WidgetScrollHandler {
    offset: Point,
}
impl WidgetScrollHandler {
    pub fn new() -> Self {
        WidgetScrollHandler { offset: Point { x: 0.0, y: 0.0 } }
    }
}
impl EventHandler for WidgetScrollHandler {
    fn event_id(&self) -> EventId {
        SCROLL_SCROLLED
    }
    fn handle_event(&mut self, args: EventArgs) {
        let EventArgs { layout, solver, .. } = args;
        let &(ref event, parent_bounds) =
            args.data.downcast_ref::<(glutin::Event, Rectangle)>().unwrap();

        let scroll = match *event {
            glutin::Event::MouseWheel(delta, _) => {
                match delta {
                    glutin::MouseScrollDelta::LineDelta(x, y) => {
                        Some(Point {
                            x: x as f64,
                            y: y as f64,
                        })
                    }
                    glutin::MouseScrollDelta::PixelDelta(x, y) => {
                        Some(Point {
                            x: x as f64,
                            y: y as f64,
                        })
                    }
                }
            }
            _ => None,
        };
        if let Some(scroll) = scroll {
            let widget_bounds = layout.bounds(solver);

            self.offset = self.offset + scroll * 13.0;
            self.offset.x = f64::min(0.0,
                                     f64::max(parent_bounds.width - widget_bounds.width,
                                              self.offset.x));
            self.offset.y = f64::min(0.0,
                                     f64::max(parent_bounds.height - widget_bounds.height,
                                              self.offset.y));
            if !solver.has_edit_variable(&layout.left) {
                solver.add_edit_variable(layout.left, STRONG).unwrap();
            }
            if !solver.has_edit_variable(&layout.top) {
                solver.add_edit_variable(layout.top, STRONG).unwrap();
            }
            solver.suggest_value(layout.left, parent_bounds.left + self.offset.x).unwrap();
            solver.suggest_value(layout.top, parent_bounds.top + self.offset.y).unwrap();
        }
        args.event_queue.push(EventAddress::Root, WIDGET_REDRAW, Box::new(()));
    }
}
