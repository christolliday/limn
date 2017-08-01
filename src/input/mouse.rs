use glutin;

use event::{Target, UiEventHandler, WidgetEventArgs};
use util::Point;
use widget::WidgetRef;
use widget::property::{Property, PropChange};
use layout::LayoutChanged;
use ui::Ui;
use app::App;

pub struct MouseMoved(pub Point);
pub struct MouseWheel(pub glutin::MouseScrollDelta);
pub struct MouseButton(pub glutin::ElementState, pub glutin::MouseButton);

#[derive(Clone)]
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
    pub widget_under_mouse: Option<WidgetRef>,
}
impl MouseController {
    pub fn new() -> Self {
        MouseController {
            mouse: Point::zero(),
            widget_under_mouse: None,
        }
    }
}
impl UiEventHandler<MouseInputEvent> for MouseController {
    fn handle(&mut self, event: &MouseInputEvent, ui: &mut Ui) {

        if let &MouseInputEvent::LayoutChanged = event {
            event!(Target::Ui, MouseMoved(Point::new(self.mouse.x, self.mouse.y)));
        }
        if let &MouseInputEvent::MouseMoved(mouse) = event {
            self.mouse = mouse;

            let widget_under_cursor = ui.widget_under_cursor(mouse);
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
        if let &MouseInputEvent::MouseButton(state, button) = event {
            if let Some(ref widget_under) = self.widget_under_mouse {
                widget_under.event_bubble_up(WidgetMouseButton(state, button));
                if (state == glutin::ElementState::Released) && (button == glutin::MouseButton::Left) {
                    let event = ClickEvent { position: self.mouse };
                    widget_under.event_bubble_up(event);
                }
            }
        }
        if let &MouseInputEvent::MouseWheel(mouse_scroll_delta) = event {
            if let Some(ref widget_under) = self.widget_under_mouse {
                widget_under.event_bubble_up(WidgetMouseWheel(mouse_scroll_delta));
            }
        }
    }
}

impl App {
    pub fn add_mouse_handlers(&mut self) {
        // adapters to create MouseInputEvents for MouseController
        self.add_handler_fn(| _: &LayoutChanged, _| {
            event!(Target::Ui, MouseInputEvent::LayoutChanged);
        });
        self.add_handler_fn(|event: &MouseMoved, _| {
            let &MouseMoved(mouse) = event;
            event!(Target::Ui, MouseInputEvent::MouseMoved(mouse));
        });
        self.add_handler_fn(|event: &MouseButton, _| {
            let &MouseButton(state, button) = event;
            event!(Target::Ui, MouseInputEvent::MouseButton(state, button));
        });
        self.add_handler_fn(|event: &MouseWheel, _| {
            let &MouseWheel(scroll) = event;
            event!(Target::Ui, MouseInputEvent::MouseWheel(scroll));
        });

        self.add_handler(MouseController::new());
    }
}

#[derive(Debug)]
pub enum MouseOverEvent {
    Over,
    Out,
}

fn handle_hover(event: &MouseOverEvent, args: WidgetEventArgs) {
    let event = match *event {
        MouseOverEvent::Over => PropChange::Add(Property::MouseOver),
        MouseOverEvent::Out => PropChange::Remove(Property::MouseOver),
    };
    args.widget.event_subtree(event);
}

impl WidgetRef {
    pub fn enable_hover(&mut self) -> &mut Self {
        self.add_handler_fn(handle_hover)
    }
}
