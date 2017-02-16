use glutin;
use cassowary::strength::*;

use event::EventAddress;
use event::events::*;
use event::id::*;
use widget::{EventArgs, EventHandler};
use util::{Point, Rectangle};

pub struct WidgetScroll {
    event: glutin::Event,
    parent_bounds: Rectangle,
}

pub struct ScrollHandler {}
impl EventHandler<WidgetMouseWheel> for ScrollHandler {
    fn handle(&mut self, event: &WidgetMouseWheel, args: EventArgs) {
        let EventArgs { widget_id, layout, event_queue, .. } = args;
        let widget_bounds = layout.bounds();
        let event = WidgetScroll { event: event.0.clone(), parent_bounds: widget_bounds };
        event_queue.push(EventAddress::Child(widget_id), NONE, event);
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
impl EventHandler<WidgetScroll> for WidgetScrollHandler {
    fn handle(&mut self, event: &WidgetScroll, args: EventArgs) {
        let EventArgs { layout, solver, .. } = args;
        let &WidgetScroll { ref event, parent_bounds } = event;

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
            let widget_bounds = layout.bounds();

            self.offset = self.offset + scroll * 13.0;
            self.offset.x = f64::min(0.0,
                                     f64::max(parent_bounds.width - widget_bounds.width,
                                              self.offset.x));
            self.offset.y = f64::min(0.0,
                                     f64::max(parent_bounds.height - widget_bounds.height,
                                              self.offset.y));
            solver.update_solver(|solver| {
                if !solver.has_edit_variable(&layout.left) {
                    solver.add_edit_variable(layout.left, STRONG).unwrap();
                }
                if !solver.has_edit_variable(&layout.top) {
                    solver.add_edit_variable(layout.top, STRONG).unwrap();
                }
                solver.suggest_value(layout.left, parent_bounds.left + self.offset.x).unwrap();
                solver.suggest_value(layout.top, parent_bounds.top + self.offset.y).unwrap();
            });
        }
    }
}
