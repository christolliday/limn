use std::collections::BTreeSet;

use widget::{EventHandler, EventArgs};
use ui::queue::EventAddress;

pub struct WidgetChangeProp {
    pub property: Property,
    pub add: bool,
}
pub struct WidgetPropsChanged;

#[derive(Hash, PartialEq, Eq, Clone, PartialOrd, Ord, Debug)]
pub enum Property {
    Hover,
    Activated,
    Selected,
    Pressed,
    Inactive,
}
pub type PropSet = BTreeSet<Property>;

pub mod states {
    use super::{Property, PropSet};
    lazy_static! {
        pub static ref STATE_DEFAULT: PropSet = btreeset!{};
        pub static ref STATE_HOVER: PropSet = btreeset!{Property::Hover};
        pub static ref STATE_PRESSED: PropSet = btreeset!{Property::Pressed};
        pub static ref STATE_ACTIVATED: PropSet = btreeset!{Property::Activated};
        pub static ref STATE_ACTIVATED_PRESSED: PropSet = btreeset!{Property::Activated, Property::Pressed};
        pub static ref STATE_SELECTED: PropSet = btreeset!{Property::Selected};
        pub static ref STATE_INACTIVE: PropSet = btreeset!{Property::Inactive};
    }
}

pub struct PropsChangeEventHandler;
impl EventHandler<WidgetChangeProp> for PropsChangeEventHandler {
    fn handle(&mut self, event: &WidgetChangeProp, mut args: EventArgs) {
        let &WidgetChangeProp { ref property, add } = event;
        if let Some(ref mut drawable) = args.widget.drawable {
            if add {
                drawable.props.insert(property.clone());
            } else {
                drawable.props.remove(property);
            }
            drawable.apply_style();
        }
        args.event_queue.push(EventAddress::Widget(args.widget.id), WidgetPropsChanged);
    }
}
