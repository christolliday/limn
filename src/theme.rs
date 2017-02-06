use std::collections::BTreeSet;
use std::collections::HashMap;

use graphics::types::Color;
use linked_hash_map::LinkedHashMap;

use widget::{Property, PropSet};
use widget::style::Value;
use widgets::primitives::RectStyle;
use widgets::text::TextStyle;
use resources::FontId;
use color::*;

static COLOR_BUTTON_DEFAULT: Color = RED;
static COLOR_BUTTON_PRESSED: Color = [0.8, 0.0, 0.0, 1.0];
static COLOR_BUTTON_ACTIVATED: Color = WHITE;
static COLOR_BUTTON_ACTIVATED_PRESSED: Color = [0.9, 0.9, 0.9, 1.0];

static COLOR_LIST_ITEM_DEFAULT: Color = [0.3, 0.3, 0.3, 1.0];
static COLOR_LIST_ITEM_HOVER: Color = [0.6, 0.6, 0.6, 1.0];
static COLOR_LIST_ITEM_SELECTED: Color = [0.2, 0.2, 1.0, 1.0];

lazy_static! {
    pub static ref STATE_DEFAULT: PropSet = btreeset!{};
    pub static ref STATE_HOVER: PropSet = btreeset!{Property::Hover};
    pub static ref STATE_PRESSED: PropSet = btreeset!{Property::Pressed};
    pub static ref STATE_ACTIVATED: PropSet = btreeset!{Property::Activated};
    pub static ref STATE_ACTIVATED_PRESSED: PropSet = btreeset!{Property::Activated, Property::Pressed};
    pub static ref STATE_SELECTED: PropSet = btreeset!{Property::Selected};

    pub static ref STYLE_LIST_ITEM: RectStyle = {
        let mut selector = LinkedHashMap::new();
        selector.insert(STATE_SELECTED.deref().clone(), COLOR_LIST_ITEM_SELECTED);
        selector.insert(STATE_HOVER.deref().clone(), COLOR_LIST_ITEM_HOVER);
        RectStyle { background: Value::Selector((selector, COLOR_LIST_ITEM_DEFAULT)) }
    };

    pub static ref STYLE_TEXT: TextStyle = {
        let text_style = Value::Single("".to_owned());
        let font_id_style = Value::Single(FontId(0)); // make first font loaded default for now
        let font_size_style = Value::Single(20.0);
        let text_color_style = Value::Single(BLACK);
        let background_color_style = Value::Single(TRANSPARENT);
        TextStyle {
            text: text_style,
            font_id: font_id_style,
            font_size: font_size_style,
            text_color: text_color_style,
            background_color: background_color_style,
        }
    };

    pub static ref STYLE_TOGGLE_BUTTON: RectStyle = {
        let mut selector = LinkedHashMap::new();
        selector.insert(STATE_ACTIVATED_PRESSED.deref().clone(), COLOR_BUTTON_ACTIVATED_PRESSED);
        selector.insert(STATE_ACTIVATED.deref().clone(), COLOR_BUTTON_ACTIVATED);
        selector.insert(STATE_PRESSED.deref().clone(), COLOR_BUTTON_PRESSED);
        selector.insert(STATE_DEFAULT.deref().clone(), COLOR_BUTTON_DEFAULT);
        RectStyle { background: Value::Selector((selector, COLOR_BUTTON_DEFAULT)) }
    };
}