use std::any::{Any, TypeId};
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

use glutin::WindowProxy;

use backend::Window;

use resources::WidgetId;
use ui::Ui;
use layout::solver::LimnSolver;
use widget::Widget;

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub enum Target {
    Widget(WidgetId),
    Child(WidgetId),
    SubTree(WidgetId),
    BubbleUp(WidgetId),
    Ui,
}

#[derive(Clone)]
pub struct Queue {
    queue: Arc<Mutex<VecDeque<(Target, TypeId, Box<Any + Send>)>>>,
    window_proxy: WindowProxy,
}

impl Queue {
    pub fn new(window: &Window) -> Self {
        Queue {
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

// allows event handlers to communicate with event dispatcher
pub struct EventState {
    pub handled: bool,
}
pub struct WidgetEventArgs<'a> {
    pub widget: &'a mut Widget,
    pub queue: &'a mut Queue,
    pub solver: &'a mut LimnSolver,
    pub event_state: &'a mut EventState,
}

pub trait WidgetEventHandler<T> {
    fn handle(&mut self, event: &T, args: WidgetEventArgs);
}

pub struct UiEventArgs<'a> {
    pub ui: &'a mut Ui,
    pub queue: &'a mut Queue,
}

pub trait UiEventHandler<T> {
    fn handle(&mut self, event: &T, args: UiEventArgs);
}

pub struct WidgetHandlerWrapper {
    type_id: TypeId,
    handler: Box<Any>,
    handle_fn: Box<Fn(&mut Box<Any>, &Box<Any + Send>, WidgetEventArgs)>,
}
impl WidgetHandlerWrapper {
    pub fn new<H, E>(handler: H) -> Self
        where H: WidgetEventHandler<E> + 'static,
              E: 'static
    {
        let handle_fn = |handler: &mut Box<Any>, event: &Box<Any + Send>, args: WidgetEventArgs| {
            let event: &E = event.downcast_ref().unwrap();
            let handler: &mut H = handler.downcast_mut().unwrap();
            handler.handle(event, args);
        };
        WidgetHandlerWrapper {
            type_id: TypeId::of::<E>(),
            handler: Box::new(handler),
            handle_fn: Box::new(handle_fn),
        }
    }
    pub fn new_from_fn<E: 'static>(handler: fn(&E, WidgetEventArgs)) -> Self {
        let handle_fn = |handler: &mut Box<Any>, event: &Box<Any + Send>, args: WidgetEventArgs| {
            let event: &E = event.downcast_ref().unwrap();
            let handler: &fn(&E, WidgetEventArgs) = handler.downcast_ref().unwrap();
            handler(event, args);
        };
        WidgetHandlerWrapper {
            type_id: TypeId::of::<E>(),
            handler: Box::new(handler),
            handle_fn: Box::new(handle_fn),
        }
    }
    pub fn handles(&self, type_id: TypeId) -> bool {
        self.type_id == type_id
    }
    pub fn handle(&mut self, event: &Box<Any + Send>, args: WidgetEventArgs) {
        (self.handle_fn)(&mut self.handler, event, args);
    }
}

pub struct UiHandlerWrapper {
    handler: Box<Any>,
    handle_fn: Box<Fn(&mut Box<Any>, &Box<Any + Send>, UiEventArgs)>,
}
impl UiHandlerWrapper {
    pub fn new<H, E>(handler: H) -> Self
        where H: UiEventHandler<E> + 'static,
              E: 'static
    {
        let handle_fn = |handler: &mut Box<Any>, event: &Box<Any + Send>, args: UiEventArgs| {
            let event: &E = event.downcast_ref().unwrap();
            let handler: &mut H = handler.downcast_mut().unwrap();
            handler.handle(event, args);
        };
        UiHandlerWrapper {
            handler: Box::new(handler),
            handle_fn: Box::new(handle_fn),
        }
    }
    pub fn new_from_fn<E: 'static>(handler: fn(&E, UiEventArgs)) -> Self {
        let handle_fn = |handler: &mut Box<Any>, event: &Box<Any + Send>, args: UiEventArgs| {
            let event: &E = event.downcast_ref().unwrap();
            let handler: &fn(&E, UiEventArgs) = handler.downcast_ref().unwrap();
            handler(event, args);
        };
        UiHandlerWrapper {
            handler: Box::new(handler),
            handle_fn: Box::new(handle_fn),
        }
    }
    pub fn handle(&mut self, event: &Box<Any + Send>, args: UiEventArgs) {
        (self.handle_fn)(&mut self.handler, event, args);
    }
}