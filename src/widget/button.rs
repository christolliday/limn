use super::EventHandler;
use super::super::event;
use event::Event;
use input::{EventId};
use std::any::Any;
use super::primitives::{RectDrawable, EllipseDrawable};
use super::text::TextDrawable;
use super::layout::WidgetLayout;
use widget::EventArgs;

use cassowary::Solver;

pub struct ToggleEventHandler {
    on: bool,
}
impl ToggleEventHandler {
    pub fn new() -> ToggleEventHandler {
        ToggleEventHandler { on: false }
    }
}
impl EventHandler for ToggleEventHandler {
    fn event_id(&self) -> EventId {
        event::WIDGET_PRESS
    }
    fn handle_event(&mut self, event_args: EventArgs) -> Option<Event> {
        let EventArgs { event, .. } = event_args;
        if let Event::Input(event) = event {
            self.on = !self.on;
            if self.on {
                Some(Event::Widget(event::Widget::ButtonEnabled(event)))
            } else {
                Some(Event::Widget(event::Widget::ButtonDisabled(event)))
            }
        } else {
            None
        }
    }
}