use backend::Window;

use ui::{self, Ui};
use event::{Queue, Target};
use ui::{EventArgs, HandlerWrapper};

pub struct App {
    pub ui: Ui,
    pub event_queue: Queue,
    pub event_handlers: Vec<HandlerWrapper>,
}

impl App {
    pub fn new(window: &mut Window) -> Self {
        let event_queue = Queue::new(window);
        let ui = Ui::new(window, &event_queue);
        App {
            ui: ui,
            event_queue: event_queue,
            event_handlers: ui::get_default_event_handlers(),
        }
    }

    pub fn render(&mut self, window: &mut Window) {
        self.ui.graph.draw_if_needed(window);
    }

    pub fn handle_events(&mut self) {
        while !self.event_queue.is_empty() {
            let (event_address, type_id, data) = self.event_queue.next();
            let data = &data;
            match event_address {
                Target::Ui => {
                    for event_handler in self.event_handlers.iter_mut() {
                        if event_handler.handles(type_id) {
                            let args = EventArgs {
                                ui: &mut self.ui,
                                event_queue: &mut self.event_queue,
                            };
                            event_handler.handle(data, args);
                        }
                    }
                }
                _ => {
                    self.ui.graph.handle_event(event_address, type_id, data, &mut self.event_queue, &mut self.ui.solver);
                }
            }
        }
    }
}