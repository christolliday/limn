use std::any::{Any, TypeId};
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

use glutin::WindowProxy;

use backend::Window;

use resources::WidgetId;

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub enum Target {
    Widget(WidgetId),
    Child(WidgetId),
    SubTree(WidgetId),
    BubbleUp(WidgetId),
    Ui,
}

#[derive(Clone)]
pub struct EventQueue {
    queue: Arc<Mutex<VecDeque<(Target, TypeId, Box<Any + Send>)>>>,
    window_proxy: WindowProxy,
}

impl EventQueue {
    pub fn new(window: &Window) -> Self {
        EventQueue {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            window_proxy: window.window.create_window_proxy(),
        }
    }
    pub fn push<T>(&mut self, address: Target, data: T)
        where T: Send + 'static
    {
        let mut queue = self.queue.lock().unwrap();
        let type_id = TypeId::of::<T>();
        queue.push_back((address, type_id, Box::new(data)));
        self.window_proxy.wakeup_event_loop();
    }
    pub fn is_empty(&mut self) -> bool {
        let queue = self.queue.lock().unwrap();
        queue.len() == 0
    }
    pub fn next(&mut self) -> (Target, TypeId, Box<Any + Send>) {
        let mut queue = self.queue.lock().unwrap();
        queue.pop_front().unwrap()
    }
}
