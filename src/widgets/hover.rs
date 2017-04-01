use event::Target;
use widget::{WidgetBuilder, EventArgs};
use widget::WidgetBuilderCore;
use widget::property::{Property, PropChange};

#[derive(Debug)]
pub enum Hover {
    Over,
    Out,
}

fn handle_hover(event: &Hover, mut args: EventArgs) {
    let event = match *event {
        Hover::Over => PropChange::Add(Property::Hover),
        Hover::Out => PropChange::Remove(Property::Hover),
    };
    args.queue.push(Target::SubTree(args.widget.id), event);
}

impl WidgetBuilder {
    pub fn enable_hover(&mut self) -> &mut Self {
        self.add_handler_fn(handle_hover)
    }
}
