use std::any::Any;

use cassowary::strength::*;

use event::{self, Event, EventAddress};
use input::{self, EventId, MouseScrollEvent};
use widget::{EventArgs, EventHandler};
use util::{Point, Rectangle};

pub const SCROLL_SCROLLED: EventId = EventId("piston/limn/scroll_scrolled");

pub struct ScrollEvent {
    pub data: (input::Event, Rectangle),
}
impl Event for ScrollEvent {
    fn event_id(&self) -> EventId {
        SCROLL_SCROLLED
    }
    fn event_data(&self) -> Option<&Any> {
        Some(&self.data)
    }
}

pub struct ScrollHandler {}
impl EventHandler for ScrollHandler {
    fn event_id(&self) -> EventId {
        event::WIDGET_SCROLL
    }
    fn handle_event(&mut self, event_args: EventArgs) {
        let EventArgs { event, widget_id, layout, event_queue, solver, .. } = event_args;
        let event = event.data::<input::Event>();
        let widget_bounds = layout.bounds(solver);
        let event = ScrollEvent { data: (event.clone(), widget_bounds) };
        event_queue.push(EventAddress::Child(widget_id), Box::new(event));
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
        let EventArgs { event, layout, solver, .. } = args;
        let &(ref event, parent_bounds) = event.data::<(input::Event, Rectangle)>();

        if let Some(scroll) = event.mouse_scroll_args() {
            let scroll: Point = scroll.into();
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
        args.state.has_updated = true;
    }
}
