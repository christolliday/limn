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

pub struct ScrollHandler {
    offset: Point,
}
impl ScrollHandler {
    pub fn new() -> Self {
        ScrollHandler { offset: Point { x: 0.0, y: 0.0 }}
    }
}
impl EventHandler for ScrollHandler {
    fn event_id(&self) -> EventId {
        event::WIDGET_SCROLL
    }
    fn handle_event(&mut self, event: Event, state: Option<&mut Any>, layout: &mut WidgetLayout, parent_layout: &WidgetLayout, solver: &mut Solver) -> Option<Event> {
        if let Event::Input(event) = event {
            Some(Event::Widget(event::Widget::ScrollScrolled(event)))
        } else {
            None
        }
    }
}

pub struct WidgetScrollHandler {
    offset: Point,
}
impl WidgetScrollHandler {
    pub fn new() -> Self {
        WidgetScrollHandler { offset: Point { x: 0.0, y: 0.0 }}
    }
}
impl EventHandler for WidgetScrollHandler {
    fn event_id(&self) -> EventId {
        event::SCROLL_SCROLLED
    }
    fn handle_event(&mut self, event: Event, state: Option<&mut Any>, layout: &mut WidgetLayout, parent_layout: &WidgetLayout, solver: &mut Solver) -> Option<Event> {
        if let Event::Widget(event) = event {
            if let event::Widget::ScrollScrolled(event) = event {
                if let Some(scroll) = event.mouse_scroll_args() {
                    let scroll: Point = scroll.into();
                    let widget_bounds = layout.bounds(solver);
                    let parent_bounds = parent_layout.bounds(solver);

                    self.offset = self.offset + scroll * 13.0;
                    self.offset.x = f64::min(0.0, f64::max(parent_bounds.width - widget_bounds.width, self.offset.x));
                    self.offset.y = f64::min(0.0, f64::max(parent_bounds.height - widget_bounds.height, self.offset.y));
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
        None
    }
}