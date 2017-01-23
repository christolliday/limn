use std::any::Any;
use std::sync::{Arc, Mutex};

use input;
use input::EventId;
use glutin::WindowProxy;

use backend::Window;

use resources::Id;

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
pub fn widget_event(event_id: EventId) -> Option<EventId> {
    match event_id {
        MOUSE_CURSOR => Some(WIDGET_MOUSE_OVER),
        MOUSE_SCROLL => Some(WIDGET_SCROLL),
        PRESS => Some(WIDGET_PRESS),
        _ => None,
    }
}

pub trait Event {
    fn event_id(&self) -> EventId;
    fn event_data(&self) -> Option<&Any>;
}
impl Event {
    pub fn data<T: 'static>(&self) -> &T {
        self.event_data().unwrap().downcast_ref::<T>().unwrap()
    }
}

// event with an id but no associated data
pub struct Signal {
    event_id: EventId,
}
impl Signal {
    pub fn new(event_id: EventId) -> Self {
        Signal { event_id: event_id }
    }
}
impl Event for Signal {
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn event_data(&self) -> Option<&Any> {
        None
    }
}

#[macro_export]
macro_rules! event {
    ( $name:ident, $data_type:path ) => {
        pub struct $name {
            event_id: EventId,
            data: $data_type,
        }
        impl $name {
            pub fn new(event_id: EventId, data: $data_type) -> Self {
                $name { event_id: event_id, data: data }
            }
        }
        impl Event for $name {
            fn event_id(&self) -> EventId {
                self.event_id
            }
            fn event_data(&self) -> Option<&Any> {
                Some(&self.data)
            }
        }
    };
}

event!(InputEvent, input::Event);

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub enum EventAddress {
    Widget(Id),
    Child(Id),
    SubTree(Id),
    UnderMouse,
}

#[derive(Clone)]
pub struct EventQueue {
    queue: Arc<Mutex<Vec<(EventAddress, Box<Event + Send>)>>>,
    window_proxy: WindowProxy,
}
impl EventQueue {
    pub fn new(window: &Window) -> Self {
        EventQueue {
            queue: Arc::new(Mutex::new(Vec::new())),
            window_proxy: window.window.window.create_window_proxy(),
        }
    }
    pub fn push(&mut self, address: EventAddress, event: Box<Event + Send>) {
        let mut queue = self.queue.lock().unwrap();
        queue.push((address, event));
        self.window_proxy.wakeup_event_loop();
    }
    pub fn is_empty(&mut self) -> bool {
        let queue = self.queue.lock().unwrap();
        queue.len() == 0
    }
    pub fn next(&mut self) -> (EventAddress, Box<Event + Send>) {
        let mut queue = self.queue.lock().unwrap();
        queue.pop().unwrap()
    }
}