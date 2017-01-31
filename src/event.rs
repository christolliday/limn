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

pub const WIDGET_HOVER: EventId = EventId("limn/widget_hover");
pub const WIDGET_SCROLL: EventId = EventId("limn/widget_scroll");
pub const WIDGET_PRESS: EventId = EventId("limn/widget_press");
pub const WIDGET_RELEASE: EventId = EventId("limn/widget_release");

pub const WIDGET_CHANGE_PROP: EventId = EventId("limn/widget_change_prop");
pub const WIDGET_PROPS_CHANGED: EventId = EventId("limn/widget_props_changed");

pub enum Hover {
    Over,
    Out,
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub enum EventAddress {
    Widget(Id),
    Child(Id),
    SubTree(Id),
    UnderMouse,
}

#[derive(Clone)]
pub struct EventQueue {
    queue: Arc<Mutex<Vec<(EventAddress, EventId, Box<Any + Send>)>>>,
    window_proxy: WindowProxy,
}
impl EventQueue {
    pub fn new(window: &Window) -> Self {
        EventQueue {
            queue: Arc::new(Mutex::new(Vec::new())),
            window_proxy: window.window.create_window_proxy(),
        }
    }
    pub fn push(&mut self, address: EventAddress, event_id: EventId, data: Box<Any + Send>) {
        let mut queue = self.queue.lock().unwrap();
        queue.push((address, event_id, data));
        self.window_proxy.wakeup_event_loop();
    }
    pub fn is_empty(&mut self) -> bool {
        let queue = self.queue.lock().unwrap();
        queue.len() == 0
    }
    pub fn next(&mut self) -> (EventAddress, EventId, Box<Any + Send>) {
        let mut queue = self.queue.lock().unwrap();
        queue.pop().unwrap()
    }
}