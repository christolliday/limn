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

    pub fn handle_events(&mut self) {
        while !self.event_queue.is_empty() {
            let (event_address, type_id, data) = self.event_queue.next();
            let data = &data;
            match event_address {
                EventAddress::Widget(id) => {
                    if let Some(node_index) = self.ui.graph.find_widget(id) {
                        self.ui.graph.trigger_widget_event(node_index, type_id, data, &mut self.event_queue, &self.ui.input_state, &mut self.ui.solver);
                    }
                }
                EventAddress::Child(id) => {
                    if let Some(node_index) = self.ui.graph.find_widget(id) {
                        if let Some(child_index) = self.ui.graph.children(node_index).next() {
                            self.ui.graph.trigger_widget_event(child_index, type_id, data, &mut self.event_queue, &self.ui.input_state, &mut self.ui.solver);
                        }
                    }
                }
                EventAddress::SubTree(id) => {
                    self.ui.graph.handle_subtree_event(id, type_id, data, &mut self.event_queue, &self.ui.input_state, &mut self.ui.solver);
                }
                EventAddress::UnderMouse => {
                    self.ui.graph.handle_undermouse_event(type_id, data, &mut self.event_queue, &mut self.ui.input_state, &mut self.ui.solver);
                }
                EventAddress::Ui => {
                    for event_handler in self.event_handlers.iter_mut() {
                        if event_handler.handles(type_id) {
                            let args = EventArgs {
                                graph: &mut self.ui.graph,
                                event_queue: &mut self.event_queue,
                                input_state: &mut self.ui.input_state,
                                solver: &mut self.ui.solver,
                            };
                            event_handler.handle(data, args);
                        }
                    }
                }
            }
        }
    }
}