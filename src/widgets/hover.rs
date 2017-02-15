use widget::{EventArgs, EventHandler};
use widget::property::Property;
use event::EventId;
use event::events::*;
use event::id::*;

pub enum Hover {
    Over,
    Out,
}

pub struct HoverHandler {}
impl EventHandler<Hover> for HoverHandler {
    fn handle(&mut self, mut args: EventArgs<Hover>) {
        let hover = args.event;
        let hover = match *hover {
            Hover::Over => true,
            Hover::Out => false,
        };

        args.event_queue.change_prop(args.widget_id, Property::Hover, hover);
    }
}
