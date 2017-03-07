use backend::Window;

use ui::{self, Ui};
use event::{Queue, Target};
use ui::{EventArgs, HandlerWrapper};

pub struct App {
    pub ui: Ui,
    pub queue: Queue,
    pub event_handlers: Vec<HandlerWrapper>,
}

impl App {
    pub fn new(window: &mut Window) -> Self {
        let queue = Queue::new(window);
        let ui = Ui::new(window, &queue);
        App {
            ui: ui,
            queue: queue,
            event_handlers: ui::get_default_event_handlers(),
        }
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
                            let args = EventArgs {
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
}