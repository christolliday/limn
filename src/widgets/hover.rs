use widget::{EventArgs, EventHandler, Property};
use event::{self, EventId, EventAddress, Hover, WIDGET_HOVER, WIDGET_CHANGE_PROP};

pub struct HoverHandler {}
impl EventHandler for HoverHandler {
    fn event_id(&self) -> EventId {
        WIDGET_HOVER
    }
    fn handle_event(&mut self, mut args: EventArgs) {
        let hover = args.data.downcast_ref::<Hover>().unwrap();
        let hover = match *hover {
            Hover::Over => true,
            Hover::Out => false,
        };
        args.event_queue.push(EventAddress::SubTree(args.widget_id), WIDGET_CHANGE_PROP, Box::new((Property::Hover, hover)));
    }
}