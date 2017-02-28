use std::collections::HashSet;

use widget::{EventArgs, EventHandler};
use widget::property::Property;
use resources::WidgetId;
use ui;
use ui::event::MouseMoved;
use ui::queue::EventAddress;

#[derive(Debug)]
pub enum Hover {
    Over,
    Out,
}

pub struct HoverHandler;
impl EventHandler<Hover> for HoverHandler {
    fn handle(&mut self, event: &Hover, mut args: EventArgs) {
        let hover = match *event {
            Hover::Over => true,
            Hover::Out => false,
        };

        args.event_queue.change_prop(args.widget_id, Property::Hover, hover);
    }
}

struct CursorChanged {
    widgets_over: Vec<WidgetId>,
}

pub struct CursorOverHandler {
    pub widgets_over: Vec<WidgetId>,
}
impl CursorOverHandler {
    pub fn new() -> Self {
        CursorOverHandler {
            widgets_over: Vec::new(),
        }
    }
}
impl ui::EventHandler<MouseMoved> for CursorOverHandler {
    fn handle(&mut self, event: &MouseMoved, args: ui::EventArgs) {
        let ui::EventArgs { ui, event_queue } = args;
        let &MouseMoved(mouse) = event;

        let mut widgets_over_changed = false;
        self.widgets_over.retain(|id| {
            let id = id.clone();
            if let Some(widget) = ui.graph.get_widget(id) {
                if !widget.is_mouse_over(mouse) {
                    event_queue.push(EventAddress::Widget(id), Hover::Out);
                    widgets_over_changed = true;
                    return false;
                }
            }
            true
        });
        let mut widgets_under_cursor = ui.graph.widgets_under_cursor(mouse);
        while let Some(widget_id) = widgets_under_cursor.next(&ui.graph.graph) {
            self.widgets_over.push(widget_id);
            widgets_over_changed = true;
            event_queue.push(EventAddress::Widget(widget_id), Hover::Over);
        }
        if widgets_over_changed {
            let event = CursorChanged {
                widgets_over: self.widgets_over.clone(),
            };
            event_queue.push(EventAddress::Ui, event);
        }
    }
}