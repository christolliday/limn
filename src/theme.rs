use graphics::types::Color;
use linked_hash_map::LinkedHashMap;

use widget::property::states::*;
use widget::style::Value;
use widgets::primitives::RectStyleField;
use color::*;

static COLOR_BUTTON_DEFAULT: Color = RED;
static COLOR_BUTTON_PRESSED: Color = [0.8, 0.0, 0.0, 1.0];
static COLOR_BUTTON_ACTIVATED: Color = WHITE;
static COLOR_BUTTON_ACTIVATED_PRESSED: Color = [0.9, 0.9, 0.9, 1.0];
static COLOR_BUTTON_INACTIVE: Color = [0.3, 0.3, 0.3, 1.0];

static COLOR_LIST_ITEM_DEFAULT: Color = [0.3, 0.3, 0.3, 1.0];
static COLOR_LIST_ITEM_HOVER: Color = [0.6, 0.6, 0.6, 1.0];
static COLOR_LIST_ITEM_SELECTED: Color = [0.2, 0.2, 1.0, 1.0];

lazy_static! {
    pub static ref STYLE_LIST_ITEM: Vec<RectStyleField> = {
        let mut selector = LinkedHashMap::new();
        selector.insert(STATE_SELECTED.deref().clone(), COLOR_LIST_ITEM_SELECTED);
        selector.insert(STATE_HOVER.deref().clone(), COLOR_LIST_ITEM_HOVER);

        vec!{ RectStyleField::BackgroundColor(Value::Selector((selector, COLOR_LIST_ITEM_DEFAULT))) }
    };

    pub static ref STYLE_BUTTON: Vec<RectStyleField> = {
        let mut selector = LinkedHashMap::new();
        selector.insert(STATE_ACTIVATED_PRESSED.deref().clone(), COLOR_BUTTON_ACTIVATED_PRESSED);
        selector.insert(STATE_ACTIVATED.deref().clone(), COLOR_BUTTON_ACTIVATED);
        selector.insert(STATE_PRESSED.deref().clone(), COLOR_BUTTON_PRESSED);
        selector.insert(STATE_INACTIVE.deref().clone(), COLOR_BUTTON_INACTIVE);
        selector.insert(STATE_DEFAULT.deref().clone(), COLOR_BUTTON_DEFAULT);

        vec!{ RectStyleField::BackgroundColor(Value::Selector((selector, COLOR_BUTTON_DEFAULT))), RectStyleField::CornerRadius(Value::Single(Some(8.0))) }
    };
}