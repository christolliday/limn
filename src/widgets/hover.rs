use widget::{EventArgs, EventHandler, WidgetProperty, ChangePropEvent};
use event::{self, EventId, EventAddress, Signal, Hover};

/*pub struct MouseOnHandler {}
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
*/

pub struct HoverHandler {}
impl EventHandler for HoverHandler {
    fn event_id(&self) -> EventId {
        event::WIDGET_HOVER
    }
    fn handle_event(&mut self, mut args: EventArgs) {
        let hover = args.event.data::<Hover>();
        let hover = match *hover {
            Hover::Over => true,
            Hover::Out => false,
        };
        let event = ChangePropEvent::new(WidgetProperty::Hover, hover);
        args.event_queue.push(EventAddress::SubTree(args.widget_id), Box::new(event));
    }
}