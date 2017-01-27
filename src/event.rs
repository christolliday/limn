use std::any::Any;
use std::sync::{Arc, Mutex};

use glutin;
use glutin::WindowProxy;

use backend::Window;

use resources::Id;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct EventId(pub &'static str);

pub const MOUSE_SCROLL: EventId = EventId("glutin/mouse_scroll");
pub const MOUSE_CURSOR: EventId = EventId("glutin/mouse_cursor");
pub const MOUSE_INPUT: EventId = EventId("glutin/mouse_input");

pub const WIDGET_MOUSE_OVER: EventId = EventId("limn/widget_mouse_over");
pub const WIDGET_MOUSE_OFF: EventId = EventId("limn/widget_mouse_off");
pub const WIDGET_SCROLL: EventId = EventId("limn/widget_scroll");
pub const WIDGET_PRESS: EventId = EventId("limn/widget_press");
pub const WIDGET_RELEASE: EventId = EventId("limn/widget_release");

pub const WIDGET_CHANGE_PROP: EventId = EventId("limn/widget_change_prop");
pub const WIDGET_PROPS_CHANGED: EventId = EventId("limn/widget_props_changed");


// get the widget event that is received if the event occurs while mouse is over widget
pub fn widget_event(event: &glutin::Event) -> Option<EventId> {
    match *event {
        glutin::Event::MouseMoved(..) => Some(WIDGET_MOUSE_OVER),
        glutin::Event::MouseWheel(..) => Some(WIDGET_SCROLL),
        glutin::Event::MouseInput(..) => Some(WIDGET_PRESS),
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

event!(InputEvent, glutin::Event);

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
            window_proxy: window.window.create_window_proxy(),
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