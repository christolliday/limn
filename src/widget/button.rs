use super::EventHandler;
use super::super::event;
use event::Event;
use input::{EventId};
use std::any::Any;
use super::primitives::RectDrawable;
use super::text::TextDrawable;
use super::layout::WidgetLayout;

use cassowary::Solver;

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
    fn handle_event(&mut self, event: Event, state: &mut Any, layout: &mut WidgetLayout, parent_layout: &WidgetLayout, solver: &mut Solver) -> Option<Event> {
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

pub struct ButtonOnHandler {}
impl EventHandler for ButtonOnHandler {
    fn event_id(&self) -> EventId {
        event::BUTTON_ENABLED
    }
    fn handle_event(&mut self, event: Event, state: &mut Any, layout: &mut WidgetLayout, parent_layout: &WidgetLayout, solver: &mut Solver) -> Option<Event> {
        if let Some(ref mut drawable) = state.downcast_mut::<RectDrawable>() {
            drawable.background = [0.0, 0.0, 0.0, 1.0];
        }
        if let Some(ref mut drawable) = state.downcast_mut::<TextDrawable>() {
            drawable.text = "ON".to_owned();
        }
        None
    }
}
pub struct ButtonOffHandler {}
impl EventHandler for ButtonOffHandler {
    fn event_id(&self) -> EventId {
        event::BUTTON_DISABLED
    }
    fn handle_event(&mut self, event: Event, state: &mut Any, layout: &mut WidgetLayout, parent_layout: &WidgetLayout, solver: &mut Solver) -> Option<Event> {
        if let Some(ref mut drawable) = state.downcast_mut::<RectDrawable>() {
            drawable.background = [1.0, 0.0, 0.0, 1.0];
        }
        if let Some(ref mut drawable) = state.downcast_mut::<TextDrawable>() {
            drawable.text = "OFF".to_owned();
        }
        None
    }
}