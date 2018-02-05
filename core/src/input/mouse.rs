//! Mouse input handlers.

use glutin;

use event::{EventHandler, EventArgs};
use geometry::Point;
use widget::Widget;
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
pub struct CursorLeftWindow;

#[derive(Debug, Copy, Clone)]
pub struct WidgetMouseWheel(pub glutin::MouseScrollDelta);
#[derive(Debug, Copy, Clone)]
pub struct WidgetMouseButton(pub glutin::ElementState, pub glutin::MouseButton);

#[derive(Debug, Copy, Clone)]
pub struct ClickEvent {
    pub position: Point,
}

#[derive(Default, Debug, Clone)]
struct MouseController {
    pub mouse: Option<Point>,
    pub widget_under_mouse: Option<Widget>,
}

impl MouseController {
    fn check_widget_under_cursor(&mut self, args: EventArgs) {
        let widget_under_mouse = self.mouse.and_then(|mouse| args.ui.widget_under_cursor(mouse));
        if widget_under_mouse != self.widget_under_mouse {
            if let Some(ref old_widget) = self.widget_under_mouse {
                old_widget.event_bubble_up(MouseOverEvent::Out);
            }
            if let Some(ref widget_under_mouse) = widget_under_mouse {
                widget_under_mouse.event_bubble_up(MouseOverEvent::Over);
            }
        }
        self.widget_under_mouse = widget_under_mouse;
    }

    fn layout_changed(&mut self, _: &LayoutChanged, args: EventArgs) {
        self.check_widget_under_cursor(args);
    }

    fn mouse_moved(&mut self, event: &MouseMoved, args: EventArgs) {
        let &MouseMoved(mouse) = event;
        self.mouse = Some(mouse);
        self.check_widget_under_cursor(args);
    }

    fn mouse_left(&mut self, _: &CursorLeftWindow, args: EventArgs) {
        self.mouse = None;
        self.check_widget_under_cursor(args);
    }

    fn mouse_button(&mut self, event: &MouseButton, _: EventArgs) {
        let &MouseButton(state, button) = event;
        if let Some(ref widget_under) = self.widget_under_mouse {
            widget_under.event_bubble_up(WidgetMouseButton(state, button));
            if (state == glutin::ElementState::Released) && (button == glutin::MouseButton::Left) && self.mouse.is_some() {
                let event = ClickEvent { position: self.mouse.unwrap() };
                widget_under.event_bubble_up(event);
            }
        }
    }

    fn mouse_wheel(&mut self, event: &MouseWheel, _: EventArgs) {
        let &MouseWheel(mouse_scroll_delta) = event;
        if let Some(ref widget_under) = self.widget_under_mouse {
            widget_under.event_bubble_up(WidgetMouseWheel(mouse_scroll_delta));
        }
    }
}

multi_event!{impl EventHandler<MouseControllerEvent> for MouseController {
    LayoutChanged => layout_changed,
    MouseMoved => mouse_moved,
    CursorLeftWindow => mouse_left,
    MouseButton => mouse_button,
    MouseWheel => mouse_wheel,
}}

impl App {
    pub fn add_mouse_handlers(&mut self) {
        self.add_handler(MouseController::default());
        MouseController::add_adapters(&mut self.get_root());
    }
}

#[derive(Debug, Copy, Clone)]
pub enum MouseOverEvent {
    Over,
    Out,
}

impl Widget {
    pub fn enable_hover(&mut self) -> &mut Self {
        self.add_handler(|event: &MouseOverEvent, mut args: EventArgs| {
            match *event {
                MouseOverEvent::Over => args.widget.add_prop(Property::MouseOver),
                MouseOverEvent::Out => args.widget.remove_prop(Property::MouseOver),
            }
        })
    }
}
