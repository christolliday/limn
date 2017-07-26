use std::any::{Any, TypeId};
use std::sync::Mutex;
use std::collections::VecDeque;

use glutin::EventsLoopProxy;

use resources::WidgetId;
use ui::Ui;
use layout::LayoutManager;
use widget::WidgetRef;

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
    /// Sends an event to a specific widget
    WidgetRef(WidgetRef),
    /// Sends an event to every descendant of a specific widget
    SubTreeRef(WidgetRef),
    /// Sends an event to a widget and continues sending to it's
    /// ancestors until an event handler marks the event as handled
    BubbleUpRef(WidgetRef),
    /// Sends an event to a UiEventHandler registered for the entire application
    Ui,
}

pub struct Queue {
    queue: VecDeque<(Target, TypeId, Box<Any>)>,
    events_loop_proxy: Option<EventsLoopProxy>,
}

impl Queue {
    fn new() -> Self {
        Queue {
            queue: VecDeque::new(),
            events_loop_proxy: None,
        }
    }
    pub fn set_events_loop(&mut self, events_loop: EventsLoopProxy) {
        self.events_loop_proxy = Some(events_loop);
    }
    /// Push a new event on the queue and wake the window up if it is asleep
    pub fn push<T: 'static>(&mut self, address: Target, data: T) {
        let type_id = TypeId::of::<T>();
        self.queue.push_back((address, type_id, Box::new(data)));
        if let Some(ref events_loop_proxy) = self.events_loop_proxy {
            events_loop_proxy.wakeup().unwrap();
        }
    }
    pub fn is_empty(&mut self) -> bool {
        self.queue.len() == 0
    }
    /// Take the next event off the Queue, should only be called by App
    pub fn next(&mut self) -> (Target, TypeId, Box<Any>) {
        self.queue.pop_front().unwrap()
    }
}

/// Context passed to a WidgetEventHandler, allows modification
/// to a widget and it's layout, and posting events to the Queue.
pub struct WidgetEventArgs<'a> {
    pub widget: WidgetRef,
    pub solver: &'a mut LayoutManager,
    pub handled: &'a mut bool,
}

/// Used to create a stateful event handler for widgets.
pub trait WidgetEventHandler<T> {
    fn handle(&mut self, event: &T, args: WidgetEventArgs);
}

/// Used to create a stateful global event handler, capable of modifying the Ui graph
pub trait UiEventHandler<T> {
    fn handle(&mut self, event: &T, ui: &mut Ui);
}

/// Non-generic WidgetEventHandler or Widget callback wrapper.
pub struct WidgetHandlerWrapper {
    handler: Box<Any>,
    handle_fn: Box<Fn(&mut Box<Any>, &Box<Any>, WidgetEventArgs)>,
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
        let handle_fn = |handler: &mut Box<Any>, event: &Box<Any>, args: WidgetEventArgs| {
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
        let handle_fn = |handler: &mut Box<Any>, event: &Box<Any>, args: WidgetEventArgs| {
            let event: &E = event.downcast_ref().unwrap();
            debug!("widget handle {}", ::type_name::<E>());
            let handler: &mut H = handler.downcast_mut().unwrap();
            handler(event, args);
        };
        WidgetHandlerWrapper {
            handler: Box::new(handler),
            handle_fn: Box::new(handle_fn),
        }
    }
    pub fn handle(&mut self, event: &Box<Any>, args: WidgetEventArgs) {
        (self.handle_fn)(&mut self.handler, event, args);
    }
}

/// Non-generic UiEventHandler or Ui callback wrapper.
pub struct UiHandlerWrapper {
    handler: Box<Any>,
    handle_fn: Box<Fn(&mut Box<Any>, &Box<Any>, &mut Ui)>,
}
impl UiHandlerWrapper {
    pub fn new<H, E>(handler: H) -> Self
        where H: UiEventHandler<E> + 'static,
              E: 'static
    {
        let handle_fn = |handler: &mut Box<Any>, event: &Box<Any>, ui: &mut Ui| {
            let event: &E = event.downcast_ref().unwrap();
            debug!("ui handle {}", ::type_name::<E>());
            let handler: &mut H = handler.downcast_mut().unwrap();
            handler.handle(event, ui);
        };
        UiHandlerWrapper {
            handler: Box::new(handler),
            handle_fn: Box::new(handle_fn),
        }
    }
    pub fn new_from_fn<H, E>(handler: H) -> Self
        where H: Fn(&E, &mut Ui) + 'static,
              E: 'static
    {
        let handle_fn = |handler: &mut Box<Any>, event: &Box<Any>, ui: &mut Ui| {
            let event: &E = event.downcast_ref().unwrap();
            let handler: &H = handler.downcast_ref().unwrap();
            handler(event, ui);
        };
        UiHandlerWrapper {
            handler: Box::new(handler),
            handle_fn: Box::new(handle_fn),
        }
    }
    pub fn handle(&mut self, event: &Box<Any>, ui: &mut Ui) {
        (self.handle_fn)(&mut self.handler, event, ui);
    }
}

lazy_static! {
    static ref FIRST_THREAD: Mutex<Cell<bool>> = Mutex::new(Cell::new(true));
}
use std::cell::{Cell, RefCell};

thread_local! {
    pub static LOCAL_QUEUE: Option<RefCell<Queue>> = {
        let first = FIRST_THREAD.lock().unwrap();
        if first.get() {
            first.set(false);
            Some(RefCell::new(Queue::new()))
        } else {
            None
        }
    }
}

pub fn queue_is_empty() -> bool {
    let mut is_empty = true;
    LOCAL_QUEUE.with(|queue| is_empty = queue.as_ref().unwrap().borrow_mut().is_empty());
    is_empty
}
pub fn queue_next() -> (Target, TypeId, Box<Any>) {
    let mut next = None;
    LOCAL_QUEUE.with(|queue| next = Some(queue.as_ref().unwrap().borrow_mut().next()));
    next.unwrap()
}
pub fn queue_set_events_loop(events_loop: EventsLoopProxy) {
    LOCAL_QUEUE.with(|queue| queue.as_ref().unwrap().borrow_mut().set_events_loop(events_loop));
}

pub fn event<T: 'static>(address: Target, data: T) {
    LOCAL_QUEUE.with(|queue| {
        if let Some(queue) = queue.as_ref() {
            debug!("push event {}", ::type_name::<T>());
            queue.borrow_mut().push(address, data);
        } else {
            eprintln!("Tried to send event off the main thread");
        }
    });
}

#[macro_export]
macro_rules! event {
    ($address:expr, $data:expr) => {
        {
            $crate::event::event($address, $data);
        }
    };
}
