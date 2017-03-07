use std::any::{Any, TypeId};

use backend::Window;

use ui::{self, Ui};
use event::{Queue, Target};

use ui::{RedrawHandler, LayoutChangeHandler};
use input::InputHandler;
use input::mouse::{MouseMoveHandler, MouseButtonHandler, MouseWheelHandler, MouseLayoutChangeHandler, MouseController};
use input::keyboard::{FocusHandler, KeyboardForwarder, KeyboardCharForwarder};
use widgets::drag::{DragInputHandler, DragMouseCursorHandler, DragMouseReleaseHandler};

pub struct App {
    pub ui: Ui,
    pub queue: Queue,
    event_handlers: Vec<UiHandlerWrapper>,
}

impl App {
    pub fn new(window: &mut Window) -> Self {
        let queue = Queue::new(window);
        let ui = Ui::new(window, &queue);
        let mut app = App {
            ui: ui,
            queue: queue,
            event_handlers: Vec::new(),
        };
        app.initialize_handlers();
        app
    }

    fn initialize_handlers(&mut self) {
        self.add_handler(RedrawHandler);
        self.add_handler(LayoutChangeHandler);
        self.add_handler(InputHandler);

        self.add_handler(MouseController::new());
        self.add_handler(MouseLayoutChangeHandler);
        self.add_handler(MouseMoveHandler);
        self.add_handler(MouseButtonHandler);
        self.add_handler(MouseWheelHandler);

        self.add_handler(KeyboardForwarder);
        self.add_handler(KeyboardCharForwarder);
        self.add_handler(FocusHandler::new());

        self.add_handler(DragInputHandler::new());
        self.add_handler(DragMouseCursorHandler);
        self.add_handler(DragMouseReleaseHandler);
    }

    pub fn render(&mut self, window: &mut Window) {
        self.ui.graph.draw_if_needed(window);
    }

    pub fn handle_events(&mut self) {
        while !self.queue.is_empty() {
            let (event_address, type_id, data) = self.queue.next();
            let data = &data;
            match event_address {
                Target::Ui => {
                    for event_handler in self.event_handlers.iter_mut() {
                        if event_handler.handles(type_id) {
                            let args = ui::EventArgs {
                                ui: &mut self.ui,
                                queue: &mut self.queue,
                            };
                            event_handler.handle(data, args);
                        }
                    }
                }
                _ => {
                    self.ui.graph.handle_event(event_address, type_id, data, &mut self.queue, &mut self.ui.solver);
                }
            }
        }
    }

    pub fn add_handler<H: ui::EventHandler<E> + 'static, E: 'static>(&mut self, handler: H) {
        self.event_handlers.push(UiHandlerWrapper::new(handler));
    }
}

struct UiHandlerWrapper {
    type_id: TypeId,
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
            type_id: TypeId::of::<E>(),
            handler: Box::new(handler),
            handle_fn: Box::new(handle_fn),
        }
    }
    pub fn handles(&self, type_id: TypeId) -> bool {
        self.type_id == type_id
    }
    pub fn handle(&mut self, event: &Box<Any + Send>, args: ui::EventArgs) {
        (self.handle_fn)(&mut self.handler, event, args);
    }
}