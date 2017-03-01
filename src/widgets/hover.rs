use widget::{EventArgs, EventHandler};
use widget::property::Property;

#[derive(Debug)]
pub enum Hover {
    Over,
    Out,
}

pub struct HoverHandler;
impl EventHandler<Hover> for HoverHandler {
    fn handle(&mut self, event: &Hover, mut args: EventArgs) {
        let hover = match *event {
            Hover::Over => true,
            Hover::Out => false,
        };

        args.event_queue.change_prop(args.widget_id, Property::Hover, hover);
    }
}
