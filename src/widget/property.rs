use std::collections::BTreeSet;

use event::EventArgs;

#[derive(Debug)]
pub enum PropChange {
    Add(Property),
    Remove(Property),
}

#[derive(Hash, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Debug)]
pub enum Property {
    MouseOver,
    Activated,
    Selected,
    Pressed,
    Inactive,
    Focused,
}
pub type PropSet = BTreeSet<Property>;

pub mod states {
    use super::{Property, PropSet};
    lazy_static! {
        pub static ref MOUSEOVER: PropSet = btreeset!{Property::MouseOver};
        pub static ref PRESSED: PropSet = btreeset!{Property::Pressed};
        pub static ref ACTIVATED: PropSet = btreeset!{Property::Activated};
        pub static ref ACTIVATED_PRESSED: PropSet = btreeset!{Property::Activated, Property::Pressed};
        pub static ref SELECTED: PropSet = btreeset!{Property::Selected};
        pub static ref INACTIVE: PropSet = btreeset!{Property::Inactive};
        pub static ref FOCUSED: PropSet = btreeset!{Property::Focused};
    }
}

pub fn prop_change_handle(event: &PropChange, mut args: EventArgs) {
    match *event {
        PropChange::Add(ref property) => args.widget.props().insert(property.clone()),
        PropChange::Remove(ref property) => args.widget.props().remove(property),
    };
    args.widget.apply_style();
}
