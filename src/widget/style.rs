use graphics::types::Color;
use linked_hash_map::LinkedHashMap;
use widget::WidgetProperty;
use std::collections::BTreeSet;

pub trait DrawableStyle<T> {
    fn apply(&self, drawable: &mut T, props: &BTreeSet<WidgetProperty>);
}

pub struct StyleSheet<T> {
    pub style_sheet: LinkedHashMap<BTreeSet<WidgetProperty>, T>,
    pub default: T,
}
impl<T: Clone> StyleSheet<T> {
    pub fn new(mut style_sheet: LinkedHashMap<BTreeSet<WidgetProperty>, T>, default: T) -> Self {
        style_sheet.insert(BTreeSet::new(), default.clone());
        StyleSheet { style_sheet: style_sheet, default: default }
    }
    pub fn apply(&self, props: &BTreeSet<WidgetProperty>) -> &T {
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

    let active_down = btreeset!{WidgetProperty::Pressed, WidgetProperty::Activated};
    let active = btreeset!{WidgetProperty::Activated};
    let down = btreeset!{WidgetProperty::Pressed};

    let mut style = LinkedHashMap::new();
    style.insert(active_down, color_active_down);
    style.insert(active, color_active);
    style.insert(down, color_down);

    let sheet = StyleSheet::new(style, color_default);

    assert_eq!(sheet.apply(&btreeset!{WidgetProperty::Activated}), &color_active);
}