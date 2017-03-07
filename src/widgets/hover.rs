use event::Target;
use widget::{EventArgs, EventHandler};
use widget::builder::WidgetBuilder;
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
        args.queue.push(Target::SubTree(args.widget.id), event);
    }
}

impl WidgetBuilder {
    pub fn enable_hover(mut self) -> Self {
        self.controller.add_handler(HoverHandler);
        self
    }
}
