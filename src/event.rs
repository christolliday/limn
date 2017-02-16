use std::any::{Any, TypeId};
use std::sync::{Arc, Mutex};

use glutin::WindowProxy;

use backend::Window;

use resources::WidgetId;
use petgraph::visit::{Dfs, DfsPostOrder};
use ui::{self, Ui};
use widget::property::{Property, WidgetChangeProp};

pub mod events {
    use glutin;
    use util::Point;

    pub struct MouseMoved(pub Point);
    pub struct MouseWheel(pub glutin::MouseScrollDelta);
    pub struct MouseButton(pub glutin::ElementState, pub glutin::MouseButton);

    pub struct WidgetMouseWheel(pub glutin::MouseScrollDelta);
    pub struct WidgetMouseButton(pub glutin::ElementState, pub glutin::MouseButton);
}


#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub enum EventAddress {
    Widget(WidgetId),
    Child(WidgetId),
    SubTree(WidgetId),
    UnderMouse,
    Ui,
}

#[derive(Clone)]
pub struct EventQueue {
    queue: Arc<Mutex<Vec<(EventAddress, TypeId, Box<Any + Send>)>>>,
    window_proxy: WindowProxy,
}

impl EventQueue {
    pub fn new(window: &Window) -> Self {
        EventQueue {
            queue: Arc::new(Mutex::new(Vec::new())),
            window_proxy: window.window.create_window_proxy(),
        }
    }
    pub fn push<T>(&mut self, address: EventAddress, data: T)
        where T: Send + 'static
    {
        let mut queue = self.queue.lock().unwrap();
        let type_id = TypeId::of::<T>();
        queue.push((address, type_id, Box::new(data)));
        self.window_proxy.wakeup_event_loop();
    }
    pub fn is_empty(&mut self) -> bool {
        let queue = self.queue.lock().unwrap();
        queue.len() == 0
    }
    pub fn next(&mut self) -> (EventAddress, TypeId, Box<Any + Send>) {
        let mut queue = self.queue.lock().unwrap();
        queue.pop().unwrap()
    }
    // common events
    pub fn change_prop(&mut self, widget_id: WidgetId, prop: Property, add: bool) {
        self.push(EventAddress::SubTree(widget_id),
                  WidgetChangeProp {
                      property: prop,
                      add: add,
                  });
    }

    pub fn handle_events(&mut self, ui: &mut Ui, ui_event_handlers: &mut Vec<ui::HandlerWrapper>) {
        while !self.is_empty() {
            let (event_address, type_id, data) = self.next();
            let data = &data;
            match event_address {
                EventAddress::Widget(id) => {
                    if let Some(node_index) = ui.find_widget(id) {
                        ui.trigger_widget_event(node_index, type_id, data, self);
                    }
                }
                EventAddress::Child(id) => {
                    if let Some(node_index) = ui.find_widget(id) {
                        if let Some(child_index) = ui.children(node_index).next() {
                            ui.trigger_widget_event(child_index, type_id, data, self);
                        }
                    }
                }
                EventAddress::SubTree(id) => {
                    if let Some(node_index) = ui.find_widget(id) {
                        let mut dfs = Dfs::new(&ui.graph, node_index);
                        while let Some(node_index) = dfs.next(&ui.graph) {
                            ui.trigger_widget_event(node_index, type_id, data, self);
                        }
                    }
                }
                EventAddress::UnderMouse => {
                    let mut dfs = DfsPostOrder::new(&ui.graph, ui.root_index.unwrap());
                    while let Some(node_index) = dfs.next(&ui.graph) {
                        let is_mouse_over = ui.is_mouse_over(node_index);
                        if is_mouse_over {
                            let handled = ui.trigger_widget_event(node_index, type_id, data, self);
                            let ref mut widget = ui.graph[node_index];
                            ui.input_state.last_over.insert(widget.id);
                            // for now just one widget can handle an event, later, just don't send to parents
                            // not no other widgets
                            if handled {
                                return;
                            }
                        }
                    }
                }
                EventAddress::Ui => {
                    for event_handler in ui_event_handlers.iter_mut() {
                        if event_handler.handles(type_id) {
                            event_handler.handle(data,
                                                 ui::EventArgs {
                                                     ui: ui,
                                                     event_queue: self,
                                                 });
                        }
                    }
                }
            }
        }
    }
}
