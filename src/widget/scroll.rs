use super::EventHandler;
use super::layout::WidgetLayout;
use event::Event;
use input;
use input::{EventId, MouseScrollEvent};
use std::any::Any;
use super::super::event;
use cassowary::{Solver, Constraint};
use cassowary::strength::*;
use util::*;
use widget::EventArgs;
use eventbus::EventAddress;

pub struct ScrollEvent {
    pub data: (input::Event, Rectangle),
}
impl Event for ScrollEvent {
    fn event_id(&self) -> EventId {
        event::SCROLL_SCROLLED
    }
    fn event_data(&self) -> Option<&Any> {
        Some(&self.data)
    }
}

pub struct ScrollHandler {
    offset: Point,
}
impl ScrollHandler {
    pub fn new() -> Self {
        ScrollHandler { offset: Point { x: 0.0, y: 0.0 } }
    }
}
impl EventHandler for ScrollHandler {
    fn event_id(&self) -> EventId {
        event::WIDGET_SCROLL
    }
    fn handle_event(&mut self, event_args: EventArgs) {
        let EventArgs { event, widget_id, layout, event_queue, solver, .. } = event_args;
        let event: &input::Event = event.event_data().unwrap().downcast_ref().unwrap();
        let widget_bounds = layout.bounds(solver);
        let event = ScrollEvent { data: (event.clone(), widget_bounds) };
        event_queue.push(EventAddress::IdAddress("CHILD".to_owned(), widget_id.0),
                         Box::new(event));
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
        event::SCROLL_SCROLLED
    }
    fn handle_event(&mut self, event_args: EventArgs) {
        let EventArgs { event, layout, solver, .. } = event_args;
        let event_data = event.event_data().unwrap();
        let &(ref event, parent_bounds) = event_data.downcast_ref::<(input::Event, Rectangle)>()
            .unwrap();

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
                solver.add_edit_variable(layout.left, STRONG);
            }
            if !solver.has_edit_variable(&layout.top) {
                solver.add_edit_variable(layout.top, STRONG);
            }
            solver.suggest_value(layout.left, parent_bounds.left + self.offset.x).unwrap();
            solver.suggest_value(layout.top, parent_bounds.top + self.offset.y).unwrap();
        }
    }
}
