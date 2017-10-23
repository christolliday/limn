use glutin;

use event::{EventHandler, EventArgs};
use geometry::Point;
use widget::{WidgetRef, WidgetBuilder};
use widget::property::Property;
use layout::LayoutChanged;
use app::App;

#[derive(Debug, Copy, Clone)]
pub struct MouseMoved(pub Point);
#[derive(Debug, Copy, Clone)]
pub struct MouseWheel(pub glutin::MouseScrollDelta);
#[derive(Debug, Copy, Clone)]
pub struct MouseButton(pub glutin::ElementState, pub glutin::MouseButton);

#[derive(Debug, Copy, Clone)]
pub struct WidgetMouseWheel(pub glutin::MouseScrollDelta);
#[derive(Debug, Copy, Clone)]
pub struct WidgetMouseButton(pub glutin::ElementState, pub glutin::MouseButton);

#[derive(Debug, Copy, Clone)]
pub enum MouseInputEvent {
    LayoutChanged,
    MouseMoved(Point),
    MouseButton(glutin::ElementState, glutin::MouseButton),
    MouseWheel(glutin::MouseScrollDelta),
}

#[derive(Debug, Copy, Clone)]
pub struct ClickEvent {
    pub position: Point,
}

#[derive(Debug, Clone)]
struct MouseController {
    pub mouse: Point,
    pub widget_under_mouse: Option<WidgetRef>,
}

impl MouseController {

    /// Creates a new MouseController
    pub fn new() -> Self {
        MouseController {
            mouse: Point::zero(),
            widget_under_mouse: None,
        }
    }

    fn check_widget_under_cursor(&mut self, args: EventArgs) {
        let widget_under_cursor = args.ui.widget_under_cursor(self.mouse);
        if widget_under_cursor != self.widget_under_mouse {
            if let Some(ref old_widget) = self.widget_under_mouse {
                old_widget.event_bubble_up(MouseOverEvent::Out);
            }
            if let Some(ref widget_under_cursor) = widget_under_cursor {
                widget_under_cursor.event_bubble_up(MouseOverEvent::Over);
            }
        }
        self.widget_under_mouse = widget_under_cursor;
    }
}
impl EventHandler<MouseInputEvent> for MouseController {
    fn handle(&mut self, event: &MouseInputEvent, args: EventArgs) {

        match *event {
            MouseInputEvent::LayoutChanged => {
                self.check_widget_under_cursor(args);
            }
            MouseInputEvent::MouseMoved(mouse) => {
                self.mouse = mouse;
                self.check_widget_under_cursor(args);
            }
            MouseInputEvent::MouseButton(state, button) => {
                if let Some(ref widget_under) = self.widget_under_mouse {
                    widget_under.event_bubble_up(WidgetMouseButton(state, button));
                    if (state == glutin::ElementState::Released) && (button == glutin::MouseButton::Left) {
                        let event = ClickEvent { position: self.mouse };
                        widget_under.event_bubble_up(event);
                    }
                }
            }
            MouseInputEvent::MouseWheel(mouse_scroll_delta) => {
                if let Some(ref widget_under) = self.widget_under_mouse {
                    widget_under.event_bubble_up(WidgetMouseWheel(mouse_scroll_delta));
                }
            }
        }
    }
}

impl App {
    pub fn add_mouse_handlers(&mut self) {
        // adapters to create MouseInputEvents for MouseController
        self.add_handler_fn(| _: &LayoutChanged, args| {
            args.widget.event(MouseInputEvent::LayoutChanged);
        });
        self.add_handler_fn(|event: &MouseMoved, args| {
            let &MouseMoved(mouse) = event;
            args.widget.event(MouseInputEvent::MouseMoved(mouse));
        });
        self.add_handler_fn(|event: &MouseButton, args| {
            let &MouseButton(state, button) = event;
            args.widget.event(MouseInputEvent::MouseButton(state, button));
        });
        self.add_handler_fn(|event: &MouseWheel, args| {
            let &MouseWheel(scroll) = event;
            args.widget.event(MouseInputEvent::MouseWheel(scroll));
        });

        self.add_handler(MouseController::new());
    }
}

#[derive(Debug, Copy, Clone)]
pub enum MouseOverEvent {
    Over,
    Out,
}

impl WidgetBuilder {
    pub fn enable_hover(&mut self) -> &mut Self {
        self.add_handler_fn(|event: &MouseOverEvent, mut args| {
            match *event {
                MouseOverEvent::Over => args.widget.add_prop(Property::MouseOver),
                MouseOverEvent::Out => args.widget.remove_prop(Property::MouseOver),
            }
        })
    }
}
