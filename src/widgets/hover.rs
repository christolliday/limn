use widget::{EventArgs, EventHandler, Property};
use event::{EventId, Hover};
use event::id::*;

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

        args.event_queue.change_prop(args.widget_id, Property::Hover, hover);
    }
}
