use super::EventHandler;
use super::super::event;
use input::{Event, EventId};
use std::any::Any;
use super::primitives::RectDrawable;

pub struct ButtonEventHandler {
    on: bool,
}
impl ButtonEventHandler {
    pub fn new() -> ButtonEventHandler {
        ButtonEventHandler { on: false }
    }
}
impl EventHandler for ButtonEventHandler {
    fn event_id(&self) -> EventId {
        event::WIDGET_PRESS
    }
    fn handle_event(&mut self, event: &Event, state: &mut Any) -> Option<EventId> {
        self.on = !self.on;
        if self.on {
            Some(event::BUTTON_ENABLED)
        } else {
            Some(event::BUTTON_DISABLED)
        }
    }
}

pub struct ButtonOnHandler {}
impl EventHandler for ButtonOnHandler {
    fn event_id(&self) -> EventId {
        event::BUTTON_ENABLED
    }
    fn handle_event(&mut self, event: &Event, state: &mut Any) -> Option<EventId> {
        let drawable: &mut RectDrawable = state.downcast_mut().unwrap();
        drawable.background = [0.0, 0.0, 0.0, 1.0];
        None
    }
}
pub struct ButtonOffHandler {}
impl EventHandler for ButtonOffHandler {
    fn event_id(&self) -> EventId {
        event::BUTTON_DISABLED
    }
    fn handle_event(&mut self, event: &Event, state: &mut Any) -> Option<EventId> {
        let drawable: &mut RectDrawable = state.downcast_mut().unwrap();
        drawable.background = [1.0, 0.0, 0.0, 1.0];
        None
    }
}