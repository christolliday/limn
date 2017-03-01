use ui;
use ui::queue::EventAddress;
use ui::event::{MouseMoved, MouseButton, MouseWheel};
use ui::event::{WidgetMouseButton, WidgetMouseWheel};
use util::Point;
use glutin;
use resources::WidgetId;
use widgets::hover::Hover;

// This contains UI event handlers used to generate events typical to a mouse controlled UI

// adapters
pub struct MouseLayoutChangeHandler;
impl ui::EventHandler<ui::LayoutChanged> for MouseLayoutChangeHandler {
    fn handle(&mut self, _: &ui::LayoutChanged, args: ui::EventArgs) {
        args.event_queue.push(EventAddress::Ui, MouseInputEvent::LayoutChanged);
    }
}
pub struct MouseMoveHandler;
impl ui::EventHandler<MouseMoved> for MouseMoveHandler {
    fn handle(&mut self, event: &MouseMoved, args: ui::EventArgs) {
        let &MouseMoved(mouse) = event;
        args.event_queue.push(EventAddress::Ui, MouseInputEvent::MouseMoved(mouse));
    }
}
pub struct MouseButtonHandler;
impl ui::EventHandler<MouseButton> for MouseButtonHandler {
    fn handle(&mut self, event: &MouseButton, args: ui::EventArgs) {
        let &MouseButton(state, button) = event;
        args.event_queue.push(EventAddress::Ui, MouseInputEvent::MouseButton(state, button));
    }
}
pub struct MouseWheelHandler;
impl ui::EventHandler<MouseWheel> for MouseWheelHandler {
    fn handle(&mut self, event: &MouseWheel, args: ui::EventArgs) {
        let &MouseWheel(scroll) = event;
        args.event_queue.push(EventAddress::Ui, MouseInputEvent::MouseWheel(scroll));
    }
}


pub enum MouseInputEvent {
    LayoutChanged,
    MouseMoved(Point),
    MouseButton(glutin::ElementState, glutin::MouseButton),
    MouseWheel(glutin::MouseScrollDelta),
}

#[derive(Clone)]
pub struct ClickEvent {
    pub position: Point,
}

pub struct MouseController {
    pub mouse: Point,
    pub widgets_over: Vec<WidgetId>,
}
impl MouseController {
    pub fn new() -> Self {
        MouseController {
            mouse: Point { x: 0.0, y: 0.0 },
            widgets_over: Vec::new(),
        }
    }
}
impl ui::EventHandler<MouseInputEvent> for MouseController {
    fn handle(&mut self, event: &MouseInputEvent, args: ui::EventArgs) {
        let ui::EventArgs { ui, event_queue } = args;

        if let &MouseInputEvent::LayoutChanged = event {
            // send new mouse event, in case widget under mouse has shifted
            let event = glutin::Event::MouseMoved(self.mouse.x as i32, self.mouse.y as i32);
            event_queue.push(EventAddress::Ui, ui::InputEvent(event));
        }
        if let &MouseInputEvent::MouseMoved(mouse) = event {
            self.mouse = mouse;
            self.widgets_over.retain(|id| {
                let id = id.clone();
                if let Some(widget) = ui.graph.get_widget(id) {
                    if !widget.is_mouse_over(mouse) {
                        event_queue.push(EventAddress::Widget(id), Hover::Out);
                        return false;
                    }
                }
                true
            });
            let mut widgets_under_cursor = ui.graph.widgets_under_cursor(mouse);
            while let Some(widget_id) = widgets_under_cursor.next(&ui.graph.graph) {
                if !self.widgets_over.contains(&widget_id) {
                    self.widgets_over.push(widget_id);
                    event_queue.push(EventAddress::Widget(widget_id), Hover::Over);
                }
            }
        }
        if let &MouseInputEvent::MouseButton(state, button) = event {
            let click_event = {
                if (state == glutin::ElementState::Released) && (button == glutin::MouseButton::Left) {
                    Some(ClickEvent { position: self.mouse })
                } else {
                    None
                }
            };
            for widget in &self.widgets_over {
                event_queue.push(EventAddress::Widget(widget.clone()), WidgetMouseButton(state, button));
                if click_event.is_some() {
                    event_queue.push(EventAddress::Widget(widget.clone()), click_event.clone());
                }
            }
        }
        if let &MouseInputEvent::MouseWheel(mouse_scroll_delta) = event {
            for widget in &self.widgets_over {
                event_queue.push(EventAddress::Widget(widget.clone()), WidgetMouseWheel(mouse_scroll_delta));
            }
        }
    }
}