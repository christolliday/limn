use std::any::{Any, TypeId};
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

use glutin::WindowProxy;

use backend::Window;

use resources::WidgetId;
use ui::Ui;
use layout::solver::LimnSolver;
use widget::Widget;

/// Defines the different targets that events can be delivered to.
/// An event will be sent to all handlers that match both the Target,
/// and the event type.
#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub enum Target {
    /// Sends an event to a specific widget
    Widget(WidgetId),
    /// Sends an event to a widgets first child
    Child(WidgetId),
    /// Sends an event to every descendant of a specific widget
    SubTree(WidgetId),
    /// Sends an event to a widget and continues sending to it's
    /// ancestors until an event handler marks the event as handled
    BubbleUp(WidgetId),
    /// Sends an event to a UiEventHandler registered for the entire application
    Ui,
}

/// The event queue, can be cloned and passed to different threads
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
    /// Push a new event on the queue and wake the window up if it is asleep
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
    /// Take the next event off the Queue, should only be called by App
    pub fn next(&mut self) -> (Target, TypeId, Box<Any + Send>) {
        let mut queue = self.queue.lock().unwrap();
        queue.pop_front().unwrap()
    }
}

/// Context passed to a WidgetEventHandler, allows modification
/// to a widget and it's layout, and posting events to the Queue.
pub struct WidgetEventArgs<'a> {
    pub widget: &'a mut Widget,
    pub queue: &'a mut Queue,
    pub solver: &'a mut LimnSolver,
    pub handled: &'a mut bool,
}

/// Used to create a stateful event handler for widgets.
pub trait WidgetEventHandler<T> {
    fn handle(&mut self, event: &T, args: WidgetEventArgs);
}

pub struct UiEventArgs<'a> {
    pub ui: &'a mut Ui,
    pub queue: &'a mut Queue,
}

/// Used to create a stateful global event handler, capable of modifying the Ui graph
pub trait UiEventHandler<T> {
    fn handle(&mut self, event: &T, args: UiEventArgs);
}

/// Non-generic WidgetEventHandler or Widget callback wrapper.
pub struct WidgetHandlerWrapper {
    handler: Box<Any>,
    handle_fn: Box<Fn(&mut Box<Any>, &Box<Any + Send>, WidgetEventArgs)>,
}
// implementation for WidgetHandlerWrapper and UiHandlerWrapper can probably only be shared
// by a macro, to make it generic over the *EventHandler and *EventArgs requires
// making *EventHandler generic over args, which is complicated by the lifetime specifier,
// could be worth revisiting if trait aliasing lands (RFC #1733)
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
            handler: Box::new(handler),
            handle_fn: Box::new(handle_fn),
        }
    }
    pub fn new_from_fn<H, E>(handler: H) -> Self
        where H: Fn(&E, WidgetEventArgs) + 'static,
              E: 'static
    {
        let handle_fn = |handler: &mut Box<Any>, event: &Box<Any + Send>, args: WidgetEventArgs| {
            let event: &E = event.downcast_ref().unwrap();
            let handler: &mut H = handler.downcast_mut().unwrap();
            handler(event, args);
        };
        WidgetHandlerWrapper {
            handler: Box::new(handler),
            handle_fn: Box::new(handle_fn),
        }
    }
    pub fn handle(&mut self, event: &Box<Any + Send>, args: WidgetEventArgs) {
        (self.handle_fn)(&mut self.handler, event, args);
    }
}

/// Non-generic UiEventHandler or Ui callback wrapper.
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
    pub fn new_from_fn<H, E>(handler: H) -> Self
        where H: Fn(&E, UiEventArgs) + 'static,
              E: 'static
    {
        let handle_fn = |handler: &mut Box<Any>, event: &Box<Any + Send>, args: UiEventArgs| {
            let event: &E = event.downcast_ref().unwrap();
            let handler: &H = handler.downcast_ref().unwrap();
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