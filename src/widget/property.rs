use std::collections::BTreeSet;

use widget::{EventHandler, EventArgs};
use event::{EventAddress, EventId};
use event::events::*;
use event::id::*;

pub struct WidgetChangeProp(pub (Property, bool));
//pub struct WidgetPropsChanged(pub glutin::Event);

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

pub struct PropsChangeEventHandler {}
impl EventHandler<WidgetChangeProp> for PropsChangeEventHandler {
    fn handle(&mut self, mut args: EventArgs<WidgetChangeProp>) {
        let (ref prop, add) = args.event.0;
        if let &mut Some(ref mut drawable) = args.drawable {
            if add {
                drawable.props.insert(prop.clone());
            } else {
                drawable.props.remove(prop);
            }
            drawable.apply_style();
        }
        args.event_queue.signal(EventAddress::Widget(args.widget_id), WIDGET_PROPS_CHANGED);
    }
}