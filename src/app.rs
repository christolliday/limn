use std::any::{Any, TypeId};
use std::collections::HashMap;

use backend::Window;

use ui::{self, Ui, RedrawHandler};
use event::{Queue, Target};

use layout::solver::LayoutChangeHandler;
use input::InputHandler;

pub struct App {
    pub ui: Ui,
    pub queue: Queue,
    handlers: HashMap<TypeId, Vec<UiHandlerWrapper>>,
}

impl App {
    pub fn new(window: &mut Window) -> Self {
        let queue = Queue::new(window);
        let ui = Ui::new(window, &queue);
        let mut app = App {
            ui: ui,
            queue: queue,
            handlers: HashMap::new(),
        };
        app.initialize_handlers();
        app
    }

    fn initialize_handlers(&mut self) {
        self.add_handler(RedrawHandler);
        self.add_handler(LayoutChangeHandler);
        self.add_handler(InputHandler);

        self.add_mouse_handlers();
        self.add_keyboard_handlers();
        self.add_drag_handlers();
    }

    pub fn render(&mut self, window: &mut Window) {
        self.ui.draw_if_needed(window);
    }

    pub fn handle_events(&mut self) {
        while !self.queue.is_empty() {
            let (event_address, type_id, data) = self.queue.next();
            let data = &data;
            match event_address {
                Target::Ui => {
                    if let Some(handlers) = self.handlers.get_mut(&type_id) {
                        for event_handler in handlers.iter_mut() {
                            let args = ui::EventArgs {
                                ui: &mut self.ui,
                                queue: &mut self.queue,
                            };
                            event_handler.handle(data, args);
                        }
                    }
                }
                _ => {
                    self.ui.handle_event(event_address, type_id, data);
                }
            }
        }
    }

    pub fn add_handler<H: ui::EventHandler<E> + 'static, E: 'static>(&mut self, handler: H) {
        let handlers = self.handlers.entry(TypeId::of::<E>()).or_insert(Vec::new());
        handlers.push(UiHandlerWrapper::new(handler));
    }
    pub fn add_handler_fn<E: 'static>(&mut self, handler: fn(&E, ui::EventArgs)) {
        let handlers = self.handlers.entry(TypeId::of::<E>()).or_insert(Vec::new());
        handlers.push(UiHandlerWrapper::new_from_fn(handler));
    }
}

struct UiHandlerWrapper {
    handler: Box<Any>,
    handle_fn: Box<Fn(&mut Box<Any>, &Box<Any + Send>, ui::EventArgs)>,
}
impl UiHandlerWrapper {
    pub fn new<H, E>(handler: H) -> Self
        where H: ui::EventHandler<E> + 'static,
              E: 'static
    {
        let handle_fn = |handler: &mut Box<Any>, event: &Box<Any + Send>, args: ui::EventArgs| {
            let event: &E = event.downcast_ref().unwrap();
            let handler: &mut H = handler.downcast_mut().unwrap();
            handler.handle(event, args);
        };
        UiHandlerWrapper {
            handler: Box::new(handler),
            handle_fn: Box::new(handle_fn),
        }
    }
    pub fn new_from_fn<E: 'static>(handler: fn(&E, ui::EventArgs)) -> Self {
        let handle_fn = |handler: &mut Box<Any>, event: &Box<Any + Send>, args: ui::EventArgs| {
            let event: &E = event.downcast_ref().unwrap();
            let handler: &fn(&E, ui::EventArgs) = handler.downcast_ref().unwrap();
            handler(event, args);
        };
        UiHandlerWrapper {
            handler: Box::new(handler),
            handle_fn: Box::new(handle_fn),
        }
    }
    pub fn handle(&mut self, event: &Box<Any + Send>, args: ui::EventArgs) {
        (self.handle_fn)(&mut self.handler, event, args);
    }
}