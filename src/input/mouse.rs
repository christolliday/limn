use glutin;

use ui;
use event::Target;
use util::Point;
use resources::WidgetId;
use widgets::hover::Hover;

use super::InputEvent;

pub struct MouseMoved(pub Point);
pub struct MouseWheel(pub glutin::MouseScrollDelta);
pub struct MouseButton(pub glutin::ElementState, pub glutin::MouseButton);

pub struct WidgetMouseWheel(pub glutin::MouseScrollDelta);
pub struct WidgetMouseButton(pub glutin::ElementState, pub glutin::MouseButton);


// adapters
pub struct MouseLayoutChangeHandler;
impl ui::EventHandler<ui::LayoutChanged> for MouseLayoutChangeHandler {
    fn handle(&mut self, _: &ui::LayoutChanged, args: ui::EventArgs) {
        args.queue.push(Target::Ui, MouseInputEvent::LayoutChanged);
    }
}
pub struct MouseMoveHandler;
impl ui::EventHandler<MouseMoved> for MouseMoveHandler {
    fn handle(&mut self, event: &MouseMoved, args: ui::EventArgs) {
        let &MouseMoved(mouse) = event;
        args.queue.push(Target::Ui, MouseInputEvent::MouseMoved(mouse));
    }
}
pub struct MouseButtonHandler;
impl ui::EventHandler<MouseButton> for MouseButtonHandler {
    fn handle(&mut self, event: &MouseButton, args: ui::EventArgs) {
        let &MouseButton(state, button) = event;
        args.queue.push(Target::Ui, MouseInputEvent::MouseButton(state, button));
    }
}
pub struct MouseWheelHandler;
impl ui::EventHandler<MouseWheel> for MouseWheelHandler {
    fn handle(&mut self, event: &MouseWheel, args: ui::EventArgs) {
        let &MouseWheel(scroll) = event;
        args.queue.push(Target::Ui, MouseInputEvent::MouseWheel(scroll));
    }
}


pub enum MouseInputEvent {
    LayoutChanged,
    MouseMoved(Point),
    MouseButton(glutin::ElementState, glutin::MouseButton),
    MouseWheel(glutin::MouseScrollDelta),
}

#[derive(Clone, Copy, Debug)]
pub struct ClickEvent {
    pub position: Point,
}

pub struct MouseController {
    pub mouse: Point,
    pub widget_under_mouse: Option<WidgetId>,
}
impl MouseController {
    pub fn new() -> Self {
        MouseController {
            mouse: Point { x: 0.0, y: 0.0 },
            widget_under_mouse: None,
        }
    }
}
impl ui::EventHandler<MouseInputEvent> for MouseController {
    fn handle(&mut self, event: &MouseInputEvent, args: ui::EventArgs) {
        let ui::EventArgs { ui, queue } = args;

        if let &MouseInputEvent::LayoutChanged = event {
            // send new mouse event, in case widget under mouse has shifted
            let event = glutin::Event::MouseMoved(self.mouse.x as i32, self.mouse.y as i32);
            queue.push(Target::Ui, InputEvent(event));
        }
        if let &MouseInputEvent::MouseMoved(mouse) = event {
            self.mouse = mouse;

            let widget_under_cursor = ui.graph.widget_under_cursor(mouse);
            if widget_under_cursor != self.widget_under_mouse {
                if let Some(old_widget) = self.widget_under_mouse {
                    queue.push(Target::BubbleUp(old_widget), Hover::Out);
                }
                if let Some(widget_under_cursor) = widget_under_cursor {
                    queue.push(Target::BubbleUp(widget_under_cursor), Hover::Over);
                }
            }
            self.widget_under_mouse = widget_under_cursor;
        }
        if let &MouseInputEvent::MouseButton(state, button) = event {
            if let Some(widget_under) = self.widget_under_mouse {
                queue.push(Target::BubbleUp(widget_under), WidgetMouseButton(state, button));
                if (state == glutin::ElementState::Released) && (button == glutin::MouseButton::Left) {
                    let event = ClickEvent { position: self.mouse };
                    queue.push(Target::BubbleUp(widget_under), event);
                }
            }
        }
        if let &MouseInputEvent::MouseWheel(mouse_scroll_delta) = event {
            if let Some(widget_under) = self.widget_under_mouse {
                queue.push(Target::BubbleUp(widget_under), WidgetMouseWheel(mouse_scroll_delta));
            }
        }
    }
}