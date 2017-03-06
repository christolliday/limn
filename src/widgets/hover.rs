use ui::queue::EventAddress;
use widget::{EventArgs, EventHandler};
use widget::property::{Property, PropChange};

#[derive(Debug)]
pub enum Hover {
    Over,
    Out,
}

pub struct HoverHandler;
impl EventHandler<Hover> for HoverHandler {
    fn handle(&mut self, event: &Hover, mut args: EventArgs) {
        let event = match *event {
            Hover::Over => PropChange::Add(Property::Hover),
            Hover::Out => PropChange::Remove(Property::Hover),
        };
        args.event_queue.push(EventAddress::SubTree(args.widget.id), event);
    }
}
