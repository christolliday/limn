use std::any::Any;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::mem;

use std::collections::HashMap;

type Id = usize;

pub struct EventBus {
    next_id: usize,
    handlers: HashMap<EventAddress, Box<Any + 'static>>,
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub enum EventAddress {
    Id(usize),
    Address(String),
    IdAddress(String, usize),
}

struct HandlerPtr<T> {
    handler: Box<Fn(T) + 'static>,
}

impl<T> HandlerPtr<T> {
    fn new(handler: Box<Fn(T) + 'static>) -> Self {
        HandlerPtr { handler: handler }
    }
}

impl EventBus {
    pub fn new() -> EventBus {
        EventBus {
            next_id: 0,
            handlers: HashMap::new(),
        }
    }

    pub fn register<T: Any, H: Fn(T) + 'static>(&mut self, handler: H) -> Id {
        let id = self.next_id;
        self.next_id = id.wrapping_add(1);

        self.register_address(EventAddress::Id(id), handler);
        id
    }

    pub fn register_address<T: Any, H: Fn(T) + 'static>(&mut self,
                                                        event_address: EventAddress,
                                                        handler: H) {
        let handler_ptr = HandlerPtr::new(Box::new(handler));
        self.handlers.insert(event_address, Box::new(handler_ptr));
    }

    pub fn unregister(&mut self, id: Id) {
        self.handlers.remove(&EventAddress::Id(id));
    }

    pub fn unregister_all(&mut self) {
        self.handlers.clear();
    }

    pub fn post_id<T: Any>(&self, id: Id, arg: T) -> bool {
        self.post_address(EventAddress::Id(id), arg)
    }

    pub fn post_address<T: Any>(&self, event_address: EventAddress, arg: T) -> bool {
        if let Some(handler) = self.handlers.get(&event_address) {
            if let Some(handler) = handler.downcast_ref::<HandlerPtr<T>>() {
                (handler.handler)(arg);
            } else {
                println!("Error, wrong event type posted for address {:?}",
                         event_address);
            }
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eventbus_works() {
        fn my_handler(arg: String) {
            println!("my_handler {:?}", arg);
        }

        let mut bus = EventBus::new();
        let handler = bus.register(my_handler);
        bus.unregister(handler);

        let string = "Hello world";
        bus.post(handler, string.to_owned());
    }
}
