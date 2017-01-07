use super::EventHandler;
use super::super::event;
use event::{Event, LimnEvent};
use input::{EventId};
use input;
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
    fn handle_event(&mut self, event_args: EventArgs) -> Option<Box<LimnEvent>> {
        let EventArgs { event, .. } = event_args;
        let event: &input::Event = event.event_data().downcast_ref().unwrap();
        
        self.on = !self.on;
        let event = if self.on {
            event::EventEvent { event: Event::Widget(event::Widget::ButtonEnabled(event.clone())) }
        } else {
            event::EventEvent { event: Event::Widget(event::Widget::ButtonDisabled(event.clone())) }
        };
        Some(Box::new(event))
    }
}
