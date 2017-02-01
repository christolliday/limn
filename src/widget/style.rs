use graphics::types::Color;
use linked_hash_map::LinkedHashMap;
use widget::{Property, PropSet};
use std::collections::BTreeSet;

#[derive(Clone)]
pub struct StyleSheet<T> {
    pub style_sheet: LinkedHashMap<PropSet, T>,
    pub default: T,
}
impl<T: Clone> StyleSheet<T> {
    pub fn new_default(default: T) -> Self {
        let mut style_sheet = LinkedHashMap::new();
        style_sheet.insert(BTreeSet::new(), default.clone());
        StyleSheet { style_sheet: style_sheet, default: default }
    }
    pub fn new(style_sheet: LinkedHashMap<PropSet, T>, default: T) -> Self {
        let mut style_sheet = style_sheet.clone();
        style_sheet.insert(BTreeSet::new(), default.clone());
        StyleSheet { style_sheet: style_sheet.clone(), default: default }
    }
    pub fn apply(&self, props: &PropSet) -> &T {
        if self.style_sheet.contains_key(&props) {
            self.style_sheet.get(&props).unwrap()
        } else {
            for (style_props, style_val) in self.style_sheet.iter() {
                // props matches all in style props
                if style_props.is_subset(&props) {
                    return style_val;
                }
            }
            &self.default
        }
    }
}

#[test]
fn test() {
    let color_active_down = [0.9, 0.9, 0.9, 1.0];
    let color_active = [1.0, 1.0, 1.0, 1.0];
    let color_down = [0.8, 0.0, 0.0, 1.0];
    let color_default = [1.0, 0.0, 0.0, 1.0];

    let active_down = btreeset!{Property::Pressed, Property::Activated};
    let active = btreeset!{Property::Activated};
    let down = btreeset!{Property::Pressed};

    let mut style = LinkedHashMap::new();
    style.insert(active_down, color_active_down);
    style.insert(active, color_active);
    style.insert(down, color_down);

    let sheet = StyleSheet::new(style, color_default);

    assert_eq!(sheet.apply(&btreeset!{Property::Activated}), &color_active);
}