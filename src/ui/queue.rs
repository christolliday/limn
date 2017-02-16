use std::any::{Any, TypeId};
use std::sync::{Arc, Mutex};

use glutin::WindowProxy;

use backend::Window;

use resources::WidgetId;
use widget::property::{Property, WidgetChangeProp};

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub enum EventAddress {
    Widget(WidgetId),
    Child(WidgetId),
    SubTree(WidgetId),
    UnderMouse,
    Ui,
}

#[derive(Clone)]
pub struct EventQueue {
    queue: Arc<Mutex<Vec<(EventAddress, TypeId, Box<Any + Send>)>>>,
    window_proxy: WindowProxy,
}

impl EventQueue {
    pub fn new(window: &Window) -> Self {
        EventQueue {
            queue: Arc::new(Mutex::new(Vec::new())),
            window_proxy: window.window.create_window_proxy(),
        }
    }
    pub fn push<T>(&mut self, address: EventAddress, data: T)
        where T: Send + 'static
    {
        let mut queue = self.queue.lock().unwrap();
        let type_id = TypeId::of::<T>();
        queue.push((address, type_id, Box::new(data)));
        self.window_proxy.wakeup_event_loop();
    }
    pub fn is_empty(&mut self) -> bool {
        let queue = self.queue.lock().unwrap();
        queue.len() == 0
    }
    pub fn next(&mut self) -> (EventAddress, TypeId, Box<Any + Send>) {
        let mut queue = self.queue.lock().unwrap();
        queue.pop().unwrap()
    }
    // common events
    pub fn change_prop(&mut self, widget_id: WidgetId, prop: Property, add: bool) {
        self.push(EventAddress::SubTree(widget_id),
                  WidgetChangeProp {
                      property: prop,
                      add: add,
                  });
    }
}
