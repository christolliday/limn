use backend::Window;

use ui::{self, Ui};
use ui::EventQueue;
use ui::{EventArgs, EventAddress, HandlerWrapper};

pub struct App {
    pub ui: Ui,
    pub event_queue: EventQueue,
    pub event_handlers: Vec<HandlerWrapper>,
}

impl App {
    pub fn new(window: &mut Window) -> Self {
        let event_queue = EventQueue::new(window);
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
                EventAddress::Ui => {
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