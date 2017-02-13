use std::collections::BTreeSet;

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