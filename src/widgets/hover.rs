use widget::{EventArgs, EventHandler, Property, ChangePropEvent};
use event::{self, EventId, EventAddress, Signal, Hover};

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
        let event = ChangePropEvent::new(Property::Hover, hover);
        args.event_queue.push(EventAddress::SubTree(args.widget_id), Box::new(event));
    }
}