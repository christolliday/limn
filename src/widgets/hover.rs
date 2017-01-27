use widget::{EventArgs, EventHandler, WidgetProperty, ChangePropEvent};
use event::{self, EventId, EventAddress, Signal};

pub struct MouseOnHandler {}
impl EventHandler for MouseOnHandler {
    fn event_id(&self) -> EventId {
        event::WIDGET_MOUSE_OVER
    }
    fn handle_event(&mut self, mut args: EventArgs) {
        let event = ChangePropEvent::new(WidgetProperty::Hover, true);
        args.event_queue.push(EventAddress::SubTree(args.widget_id), Box::new(event));
    }
}
pub struct MouseOffHandler {}
impl EventHandler for MouseOffHandler {
    fn event_id(&self) -> EventId {
        event::WIDGET_MOUSE_OFF
    }
    fn handle_event(&mut self, mut args: EventArgs) {
        let event = ChangePropEvent::new(WidgetProperty::Hover, false);
        args.event_queue.push(EventAddress::SubTree(args.widget_id), Box::new(event));
    }
}