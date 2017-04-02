use glutin;

use event::{Target, UiEventHandler, UiEventArgs};
use util::Point;
use resources::WidgetId;
use widgets::hover::Hover;
use layout::solver::LayoutChanged;
use app::App;

use super::InputEvent;

pub struct MouseMoved(pub Point);
pub struct MouseWheel(pub glutin::MouseScrollDelta);
pub struct MouseButton(pub glutin::ElementState, pub glutin::MouseButton);

pub struct WidgetMouseWheel(pub glutin::MouseScrollDelta);
pub struct WidgetMouseButton(pub glutin::ElementState, pub glutin::MouseButton);

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

struct MouseController {
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
impl UiEventHandler<MouseInputEvent> for MouseController {
    fn handle(&mut self, event: &MouseInputEvent, args: UiEventArgs) {
        let UiEventArgs { ui, queue } = args;

        if let &MouseInputEvent::LayoutChanged = event {
            // send new mouse event, in case widget under mouse has shifted
            let event = glutin::Event::MouseMoved(self.mouse.x as i32, self.mouse.y as i32);
            queue.push(Target::Ui, InputEvent(event));
        }
        if let &MouseInputEvent::MouseMoved(mouse) = event {
            self.mouse = mouse;

            let widget_under_cursor = ui.widget_under_cursor(mouse);
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

impl App {
    pub fn add_mouse_handlers(&mut self) {
        // adapters to create MouseInputEvents for MouseController
        self.add_handler_fn(| _: &LayoutChanged, args| {
            args.queue.push(Target::Ui, MouseInputEvent::LayoutChanged);
        });
        self.add_handler_fn(|event: &MouseMoved, args| {
            let &MouseMoved(mouse) = event;
            args.queue.push(Target::Ui, MouseInputEvent::MouseMoved(mouse));
        });
        self.add_handler_fn(|event: &MouseButton, args| {
            let &MouseButton(state, button) = event;
            args.queue.push(Target::Ui, MouseInputEvent::MouseButton(state, button));
        });
        self.add_handler_fn(|event: &MouseWheel, args| {
            let &MouseWheel(scroll) = event;
            args.queue.push(Target::Ui, MouseInputEvent::MouseWheel(scroll));
        });

        self.add_handler(MouseController::new());
    }
}