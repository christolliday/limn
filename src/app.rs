use std::any::TypeId;
use std::collections::HashMap;

use backend::{Window, WindowEvents};
use backend::events::WindowEvent;
use glutin::Event;

use ui::Ui;
use input::{self, InputEvent};
use event::{Queue, Target, UiHandlerWrapper, UiEventHandler, UiEventArgs};
use layout::solver;
use util::Dimensions;

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
    pub fn resize_window_to_fit(&mut self, window: &Window) {
        // handle layout change events, needed to measure widgets before resizing window
        self.handle_events();
        self.ui.resize_window_to_fit(&window);
    }

    fn initialize_handlers(&mut self) {
        self.add_handler_fn(solver::handle_layout_change);
        self.add_handler_fn(input::handle_input);

        self.add_mouse_handlers();
        self.add_keyboard_handlers();
        self.add_drag_handlers();
    }

    pub fn main_loop(&mut self, window: &mut Window) {
        let mut events = WindowEvents::new();
        while !self.ui.should_close() {
            let event = events.next(&mut window.window);
            match event {
                WindowEvent::Input(event) => {
                    match event {
                        Event::Resized(width, height) => {
                            window.window_resized();
                            self.ui.window_resized(Dimensions {
                                width: width as f64,
                                height: height as f64,
                            });
                        }
                        Event::Awakened => {
                            self.handle_events();
                        }
                        _ => {
                            self.queue.push(Target::Ui, InputEvent(event));
                            self.handle_events();
                        },
                    }
                }
                WindowEvent::Render => {
                    self.ui.draw_if_needed(window);
                }
            }
        }
    }

    pub fn handle_events(&mut self) {
        while !self.queue.is_empty() {
            let (event_address, type_id, data) = self.queue.next();
            let data = &data;
            match event_address {
                Target::Ui => {
                    if let Some(handlers) = self.handlers.get_mut(&type_id) {
                        for event_handler in handlers.iter_mut() {
                            let args = UiEventArgs {
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

    pub fn add_handler<H: UiEventHandler<E> + 'static, E: 'static>(&mut self, handler: H) {
        self.handlers.entry(TypeId::of::<E>()).or_insert(Vec::new())
            .push(UiHandlerWrapper::new(handler));
    }
    pub fn add_handler_fn<E: 'static>(&mut self, handler: fn(&E, UiEventArgs)) {
        self.handlers.entry(TypeId::of::<E>()).or_insert(Vec::new())
            .push(UiHandlerWrapper::new_from_fn(handler));
    }
}
