use input;
use input::EventId;
use input::GenericEvent;

use std::any::Any;

// from piston input
pub const AFTER_RENDER: EventId = EventId("piston/after_render");
pub const CONTROLLER_AXIS: EventId = EventId("piston/controller_axis");
pub const CURSOR: EventId = EventId("piston/cursor");
pub const FOCUS: EventId = EventId("piston/focus");
pub const CLOSE: EventId = EventId("piston/close");
pub const IDLE: EventId = EventId("piston/idle");
pub const MOUSE_SCROLL: EventId = EventId("piston/mouse_scroll");
pub const MOUSE_RELATIVE: EventId = EventId("piston/mouse_relative");
pub const MOUSE_CURSOR: EventId = EventId("piston/mouse_cursor");
pub const PRESS: EventId = EventId("piston/press");
pub const RELEASE: EventId = EventId("piston/release");
pub const RENDER: EventId = EventId("piston/render");
pub const RESIZE: EventId = EventId("piston/resize");
pub const TEXT: EventId = EventId("piston/text");
pub const TOUCH: EventId = EventId("piston/touch");
pub const UPDATE: EventId = EventId("piston/update");

pub const WIDGET_MOUSE_OVER: EventId = EventId("piston/limn/widget_mouse_over");
pub const WIDGET_SCROLL: EventId = EventId("piston/limn/widget_scroll");
pub const WIDGET_PRESS: EventId = EventId("piston/limn/widget_press");
pub const WIDGET_RELEASE: EventId = EventId("piston/limn/widget_release");

pub const SCROLL_SCROLLED: EventId = EventId("piston/limn/scroll_scrolled");
pub const BUTTON_ENABLED: EventId = EventId("piston/limn/button_enabled");
pub const BUTTON_DISABLED: EventId = EventId("piston/limn/button_disabled");

// get the widget event that is received if the event occurs while mouse is over widget
pub fn widget_event<E: LimnEvent>(event: &E) -> Option<EventId> {
    match event.event_id() {
        MOUSE_CURSOR => Some(WIDGET_MOUSE_OVER),
        MOUSE_SCROLL => Some(WIDGET_SCROLL),
        PRESS => Some(WIDGET_PRESS),
        _ => None
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Widget {
    // widget general, received when mouse over widget
    MouseOver(input::Event),
    Scroll(input::Event),
    Press(input::Event),
    Release(input::Event),

    // specific widgets
    ScrollScrolled(input::Event),
    ButtonEnabled(input::Event),
    ButtonDisabled(input::Event),
}

#[derive(Clone, PartialEq, Debug)]
pub enum Event {
    Input(input::Event),
    Widget(Widget),
}
impl Event {
    pub fn event_id(&self) -> EventId {
        match *self {
            Event::Input(ref event) => event.event_id(),
            Event::Widget(ref event) => {
                match *event {
                    Widget::MouseOver(_) => WIDGET_MOUSE_OVER,
                    Widget::Scroll(_) => WIDGET_SCROLL,
                    Widget::Press(_) => WIDGET_PRESS,
                    Widget::Release(_) => WIDGET_RELEASE,
                    
                    Widget::ScrollScrolled(_) => SCROLL_SCROLLED,
                    Widget::ButtonEnabled(_) => BUTTON_ENABLED,
                    Widget::ButtonDisabled(_) => BUTTON_DISABLED,
                }
            },
        }
    }
}

pub trait LimnEvent {
    fn event_id(&self) -> EventId;
    fn event_data(&self) -> &Any;
}
pub struct InputEvent {
    pub event: input::Event,
}
impl LimnEvent for InputEvent {
    fn event_id(&self) -> EventId {
        self.event.event_id()
    }
    fn event_data(&self) -> &Any {
        &self.event
    }
}
pub struct EventEvent {
    pub event: Event,
}
impl LimnEvent for EventEvent {
    fn event_id(&self) -> EventId {
        self.event.event_id()
    }
    fn event_data(&self) -> &Any {
        &self.event
    }
}