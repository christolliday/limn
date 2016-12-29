use input;
use input::{Input, Motion, Button};
use input::EventId;
use input::GenericEvent;

// from piston input
pub const AFTER_RENDER: EventId = EventId("piston/after_render");
pub const CONTROLLER_AXIS: EventId = EventId("piston/controller_axis");
pub const CURSOR: EventId = EventId("piston/cursor");
pub const FOCUS: EventId = EventId("piston/focus");
pub const CLOSE: EventId = EventId("piston/close");
pub const IDLE: EventId = EventId("piston/idle");
pub const MOUSE_SCROLL: EventId = EventId("piston/mouse_scroll");
pub const MOUSE_RELATIVE: EventId = EventId("piston/mouse_relative");
pub const MOUSE_CURSOR: EventId = EventId("piston/mouse_cursor");
pub const PRESS: EventId = EventId("piston/press");
pub const RELEASE: EventId = EventId("piston/release");
pub const RENDER: EventId = EventId("piston/render");
pub const RESIZE: EventId = EventId("piston/resize");
pub const TEXT: EventId = EventId("piston/text");
pub const TOUCH: EventId = EventId("piston/touch");
pub const UPDATE: EventId = EventId("piston/update");

pub const WIDGET_MOUSE_OVER: EventId = EventId("piston/limn/widget_mouse_over");
pub const WIDGET_PRESS: EventId = EventId("piston/limn/widget_press");
pub const WIDGET_RELEASE: EventId = EventId("piston/limn/widget_release");
pub const WIDGET_SCROLL: EventId = EventId("piston/limn/widget_scroll");

pub const BUTTON_ENABLED: EventId = EventId("piston/limn/button_enabled");
pub const BUTTON_DISABLED: EventId = EventId("piston/limn/button_disabled");

/// Events that apply to a specific widget.
///
/// Rather than delivering entire `event::Event`s to the widget (with a lot of redundant
/// information), this `event::Widget` is used as a refined, widget-specific event.
///
/// All `Widget` event co-ordinates will be relative to the centre of the `Widget` to which they
/// are delivered.
#[derive(Clone, PartialEq, Debug)]
pub enum Widget {
    MouseOver(input::Event),
    Scroll(input::Event),
    /// Some button was pressed.
    Press(input::Event),
    /// Some button was released.
    Release(input::Event),
    Enabled(input::Event),
    Disabled(input::Event),
}

#[derive(Clone, PartialEq, Debug)]
pub enum Event {
    Input(input::Event),
    Widget(Widget),
}
impl Event {
    pub fn event_id(&self) -> EventId {
        match *self {
            Event::Input(ref event) => event.event_id(),
            Event::Widget(ref event) => {
                match *event {
                    Widget::MouseOver(_) => WIDGET_MOUSE_OVER,
                    Widget::Scroll(_) => WIDGET_SCROLL,
                    Widget::Press(_) => WIDGET_PRESS,
                    Widget::Release(_) => WIDGET_RELEASE,
                    Widget::Enabled(_) => BUTTON_ENABLED,
                    Widget::Disabled(_) => BUTTON_DISABLED,
                }
            },
        }
    }
}