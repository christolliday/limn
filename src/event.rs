use input::{Motion, Button};
use input::EventId;

pub const WIDGET_MOUSE_OVER: EventId = EventId("piston/limn/widget_mouse_over");
pub const WIDGET_PRESS: EventId = EventId("piston/limn/widget_press");
pub const WIDGET_RELEASE: EventId = EventId("piston/limn/widget_release");

/// Events that apply to a specific widget.
///
/// Rather than delivering entire `event::Event`s to the widget (with a lot of redundant
/// information), this `event::Widget` is used as a refined, widget-specific event.
///
/// All `Widget` event co-ordinates will be relative to the centre of the `Widget` to which they
/// are delivered.
#[derive(Clone, PartialEq, Debug)]
pub enum Widget {
    MouseOver(Motion),
    /// Some button was pressed.
    Press(Button),
    /// Some button was released.
    Release(Button),
}
