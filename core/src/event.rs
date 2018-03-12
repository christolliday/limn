//! Contains types relevant to event handling and the event queue.
//!
//! The event system in limn is based on asynchronous message passing between the event handlers contained in widgets.
//! By creating types implementing `EventHandler` you can define reusable behaviour that can be applied to any widget,
//! or any application.
//!
//! Handlers can be added to a widget using `Widget::add_handler`, or to the root widget using `Ui::add_handler`.
//! Typically handlers in the root widget are used to interface with the outside world or manage application global
//! state. Input events are always sent to the root widget first, which has handlers that can redirect them to the
//! appropriate widgets, the widget under the mouse, or the widget that has keyboard focus, for example.
//!
//! There are different ways events can be dispatched:
//!
//! - `Widget::event`
//!
//!   send an event to a single widget.
//! - `Widget::event_subtree`
//!
//!   send an event to a widget and recursively send it to all it's children.
//! - `Widget::event_bubble_up`
//!
//!   send an event to a widget, then send it to the widgets parent either until you
//! reach the root or some widget marks it as handled.
//! - `Ui::event`
//!
//!   send an event to the root widget. This is purely a convenience method, which removes the need to pass references to
//!   the root around, since `Ui` is available as an argument to every handle method.
//!
//! Currently, `limn` handles all widget events on a single thread, the UI thread, which is the only thread that can modify
//! UI state. This means to keep your app responsive, any event handler that needs to block or do long running work must do
//! it on another thread, either by spawning or notifying a thread, which can then send an event back to the UI when it's
//! ready. The `event_global` helper method makes this easier, it is equivalent to `Ui::event` but requires the event be
//! `Send` and can be called from any thread, without a reference to the `Ui`. `Widget` and any other types that can
//! modify the UI are not thread safe, so can't currently be referenced from other threads, so if any specific widgets need
//! to be notified from another thread, it's necessary to add a handler to the root widget to forward events.
//!
//! For further explanation of the single threaded event architecture see
//! https://github.com/christolliday/limn/pull/20#discussion_r145373568

use std::any::{Any, TypeId};
use std::cell::{Cell, RefCell};
use std::sync::Mutex;
use std::collections::VecDeque;

use glutin::{EventsLoop, EventsLoopProxy};

use ui::Ui;
use widget::Widget;

/// Defines the different targets that events can be delivered to.
/// An event will be sent to all handlers that match both the Target,
/// and the event type.
#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub(crate) enum Target {
    /// Sends an event to a specific widget
    Widget(Widget),
    /// Sends an event to every descendant of a specific widget
    SubTree(Widget),
    /// Sends an event to a widget and continues sending to it's
    /// ancestors until an event handler marks the event as handled
    BubbleUp(Widget),
    /// Sends an event to the root widget
    Root,
}

struct Queue {
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
    fn set_events_loop(&mut self, events_loop: EventsLoopProxy) {
        self.events_loop_proxy = Some(events_loop);
    }
    /// Push a new event on the queue and wake the window up if it is asleep
    fn push<T: 'static>(&mut self, address: Target, data: T) {
        let type_id = TypeId::of::<T>();
        self.queue.push_back((address, type_id, Box::new(data)));
        if let Some(ref events_loop_proxy) = self.events_loop_proxy {
            events_loop_proxy.wakeup().unwrap();
        }
    }
}
impl Iterator for Queue {
    type Item = (Target, TypeId, Box<Any>);
    /// Take the next event off the Queue, should only be called by App
    fn next(&mut self) -> Option<(Target, TypeId, Box<Any>)> {
        self.queue.pop_front()
    }
}

/// Context passed to a `EventHandler`, provides access to the widget
/// that holds it, the `Ui`, and a flag to notify the dispatcher that
/// the event has been handled (in the case the event is bubbling up)
pub struct EventArgs<'a> {
    pub widget: Widget,
    pub ui: &'a mut Ui,
    pub handled: &'a mut bool,
}

/// Used to create a stateful event handler for widgets.
pub trait EventHandler<T> {
    fn handle(&mut self, event: &T, args: EventArgs);
}

impl <T, E> EventHandler<E> for T where T: FnMut(&E, EventArgs) {
    fn handle(&mut self, event: &E, args: EventArgs) {
        self(event, args);
    }
}

/// Non-generic `EventHandler` or Widget callback wrapper.
pub(super) struct EventHandlerWrapper {
    handler: Box<Any>,
    handle_fn: Box<Fn(&mut Any, &Any, EventArgs)>,
}

impl EventHandlerWrapper {
    pub fn new<H, E>(handler: H) -> Self
        where H: EventHandler<E> + 'static,
              E: 'static
    {
        let handle_fn = |handler: &mut Any, event: &Any, args: EventArgs| {
            let event: &E = event.downcast_ref().unwrap();
            let handler: &mut H = handler.downcast_mut().unwrap();
            handler.handle(event, args);
        };
        EventHandlerWrapper {
            handler: Box::new(handler),
            handle_fn: Box::new(handle_fn),
        }
    }
    pub fn new_from_fn<H, E>(handler: H) -> Self
        where H: FnMut(&E, EventArgs) + 'static,
              E: 'static
    {
        let handle_fn = |handler: &mut Any, event: &Any, args: EventArgs| {
            let event: &E = event.downcast_ref().unwrap();
            debug!("widget handle {}", ::type_name::<E>());
            let handler: &mut H = handler.downcast_mut().unwrap();
            handler(event, args);
        };
        EventHandlerWrapper {
            handler: Box::new(handler),
            handle_fn: Box::new(handle_fn),
        }
    }
    pub fn handle(&mut self, event: &Any, args: EventArgs) {
        (self.handle_fn)(self.handler.as_mut(), event, args);
    }
}

lazy_static! {
    static ref FIRST_THREAD: Mutex<Cell<bool>> = Mutex::new(Cell::new(true));
    static ref GLOBAL_QUEUE: Mutex<GlobalQueue> = Mutex::new(GlobalQueue::new());
}

thread_local! {
    static LOCAL_QUEUE: Option<RefCell<Queue>> = {
        let first = FIRST_THREAD.lock().unwrap();
        if first.get() {
            first.set(false);
            Some(RefCell::new(Queue::new()))
        } else {
            None
        }
    }
}

pub(super) fn queue_next() -> Option<(Target, TypeId, Box<Any>)> {
    if let Some(next) = GLOBAL_QUEUE.lock().unwrap().next() {
        Some((Target::Root, next.0, next.1))
    } else {
        let mut next = None;
        LOCAL_QUEUE.with(|queue| next = Some(queue.as_ref().unwrap().borrow_mut().next()));
        next.unwrap()
    }
}

pub(super) fn queue_set_events_loop(events_loop: &EventsLoop) {
    GLOBAL_QUEUE.lock().unwrap().set_events_loop(events_loop.create_proxy());
    LOCAL_QUEUE.with(|queue| queue.as_ref().unwrap().borrow_mut().set_events_loop(events_loop.create_proxy()));
}

/// Send message to target address, must be sent from main UI thread.
pub(crate) fn event<T: 'static>(address: Target, data: T) {
    LOCAL_QUEUE.with(|queue| {
        if let Some(queue) = queue.as_ref() {
            debug!("push event {}", ::type_name::<T>());
            queue.borrow_mut().push(address, data);
        } else {
            eprintln!("Tried to send event off the main thread, use event_global");
        }
    });
}

/// Send message to UI from any thread.
pub fn event_global<T: 'static + Send>(data: T) {
    GLOBAL_QUEUE.lock().unwrap().push(data);
}

struct GlobalQueue {
    queue: VecDeque<(TypeId, Box<Any + Send>)>,
    events_loop_proxy: Option<EventsLoopProxy>,
}

impl GlobalQueue {
    fn new() -> Self {
        GlobalQueue {
            queue: VecDeque::new(),
            events_loop_proxy: None,
        }
    }
    pub fn set_events_loop(&mut self, events_loop: EventsLoopProxy) {
        self.events_loop_proxy = Some(events_loop);
    }
    /// Push a new event on the queue and wake the window up if it is asleep
    pub fn push<T: 'static + Send>(&mut self, data: T) {
        let type_id = TypeId::of::<T>();
        self.queue.push_back((type_id, Box::new(data)));
        if let Some(ref events_loop_proxy) = self.events_loop_proxy {
            events_loop_proxy.wakeup().unwrap();
        }
    }
}
impl Iterator for GlobalQueue {
    type Item = (TypeId, Box<Any + Send>);
    fn next(&mut self) -> Option<(TypeId, Box<Any + Send>)> {
        self.queue.pop_front()
    }
}

/// Simplifies setting up an EventHandler that can receive multiple events. Generates an event enum and matches each event to a method on the handler.
/// Also creates an associated method on the handler, `add_adapters` that should be called when the handler is added to a widget, to add the "adapter"
/// handlers that redirect each event to the main event handler.
#[macro_export]
macro_rules! multi_event {
    ( impl EventHandler< $multi_event:ident > for $handler:ident { $ ( $event_type:ident => $event_method:ident, ) * } ) => {
        enum $multi_event {
            $(
                $event_type($event_type),
            )*
        }
        impl $handler {
            #[allow(dead_code)]
            fn add_adapters(widget: &mut Widget) {
                $(
                    widget.add_handler(|event: &$event_type, args: EventArgs| {
                        args.widget.event($multi_event::$event_type(event.clone()));
                    });
                )*
            }
        }
        impl EventHandler<$multi_event> for $handler {
            fn handle(&mut self, event: &$multi_event, args: EventArgs) {
                match *event { $(
                    $multi_event::$event_type(ref event) => self.$event_method(event, args),
                )* }
            }
        }
    }
}

/// Specifies a handler that redirects events from one widget to another.
/// Optionally you can specify a closure that modifies the event.
#[macro_export]
macro_rules! forward_event {
    ( $event:ident : $source:ident -> $multi_event:ident : $destination:ident ) => {
        let destination = $destination.clone();
        $source.add_handler(move |event: &$event, _: EventArgs| {
            destination.event($multi_event::$event(event.clone()));
        });
    };
    ( $event:ty : $closure:expr ; $source:ident -> $destination:ident ) => {
        let closure: Box<Fn(&$event, EventArgs) -> _> = Box::new($closure);
        let destination = $destination.clone();
        $source.add_handler(move |event: &$event, args: EventArgs| {
            destination.event(closure(event, args));
        });
    }
}
