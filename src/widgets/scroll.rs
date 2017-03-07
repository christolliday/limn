use glutin;
use cassowary::strength::*;

use event::Target;
use widget::{EventArgs, EventHandler};
use util::{Point, Rectangle};

use input::mouse::WidgetMouseWheel;

pub struct WidgetScroll {
    event: glutin::MouseScrollDelta,
    parent_bounds: Rectangle,
}

pub struct ScrollHandler;
impl EventHandler<WidgetMouseWheel> for ScrollHandler {
    fn handle(&mut self, event: &WidgetMouseWheel, args: EventArgs) {
        let EventArgs { widget, event_queue, .. } = args;
        let widget_bounds = widget.layout.bounds();
        let event = WidgetScroll {
            event: event.0,
            parent_bounds: widget_bounds,
        };
        event_queue.push(Target::Child(widget.id), event);
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
        let EventArgs { widget, solver, .. } = args;
        let ref layout = widget.layout;
        let &WidgetScroll { event, parent_bounds } = event;

        let scroll = match event {
            glutin::MouseScrollDelta::LineDelta(x, y) => {
                Point {
                    x: x as f64,
                    y: y as f64,
                }
            }
            glutin::MouseScrollDelta::PixelDelta(x, y) => {
                Point {
                    x: x as f64,
                    y: y as f64,
                }
            }
        };

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
