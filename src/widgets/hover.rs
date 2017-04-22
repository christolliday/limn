use event::{Target, WidgetEventArgs};
use widget::{WidgetBuilder, WidgetBuilderCore};
use widget::property::{Property, PropChange};

#[derive(Debug)]
pub enum Hover {
    Over,
    Out,
}

fn handle_hover(event: &Hover, args: WidgetEventArgs) {
    let event = match *event {
        Hover::Over => PropChange::Add(Property::Hover),
        Hover::Out => PropChange::Remove(Property::Hover),
    };
    event!(Target::SubTree(args.widget.id), event);
}

impl WidgetBuilder {
    pub fn enable_hover(&mut self) -> &mut Self {
        self.add_handler_fn(handle_hover)
    }
}
