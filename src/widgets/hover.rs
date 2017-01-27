use widget::{EventArgs, EventHandler, WidgetProperty};
use event::{self, EventId, EventAddress, Signal};

pub struct MouseOnHandler {}
impl EventHandler for MouseOnHandler {
    fn event_id(&self) -> EventId {
        event::WIDGET_MOUSE_OVER
    }
    fn handle_event(&mut self, mut args: EventArgs) {
        args.props.insert(WidgetProperty::Hover);
        args.event_queue.push(EventAddress::Widget(args.widget_id), Box::new(Signal::new(event::WIDGET_PROPS_CHANGED)));
    }
}
pub struct MouseOffHandler {}
impl EventHandler for MouseOffHandler {
    fn event_id(&self) -> EventId {
        event::WIDGET_MOUSE_OFF
    }
    fn handle_event(&mut self, mut args: EventArgs) {
        args.props.remove(&WidgetProperty::Hover);
        args.event_queue.push(EventAddress::Widget(args.widget_id), Box::new(Signal::new(event::WIDGET_PROPS_CHANGED)));
    }
}