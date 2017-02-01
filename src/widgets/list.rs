use linked_hash_map::LinkedHashMap;
use graphics::types::Color;

use widget::{EventArgs, EventHandler, Property, PropSet};
use widgets::primitives::RectStyle;
use widget::style::Value;
use event::{self, EventId, EventAddress, WIDGET_CHANGE_PROP};
use resources::Id;
use color::*;

const WIDGET_LIST_ITEM_SELECTED: EventId = EventId("limn/list_item_selected");

static COLOR_LIST_ITEM_DEFAULT: Color = [0.3, 0.3, 0.3, 1.0];
static COLOR_LIST_ITEM_HOVER: Color = [0.6, 0.6, 0.6, 1.0];
static COLOR_LIST_ITEM_SELECTED: Color = [0.2, 0.2, 1.0, 1.0];

lazy_static! {
    pub static ref STATE_DEFAULT: PropSet = btreeset!{};
    pub static ref STATE_SELECTED: PropSet = btreeset!{Property::Selected};
    pub static ref STATE_HOVER: PropSet = btreeset!{Property::Hover};
    pub static ref LIST_ITEM_STYLE_DEFAULT: RectStyle = {
        let mut selector = LinkedHashMap::new();
        selector.insert(STATE_SELECTED.deref().clone(), COLOR_LIST_ITEM_SELECTED);
        selector.insert(STATE_HOVER.deref().clone(), COLOR_LIST_ITEM_HOVER);
        RectStyle { background: Value::Selector((selector, COLOR_LIST_ITEM_DEFAULT)) }
    };
}

pub struct ListHandler {
    selected: Option<Id>,
}
impl ListHandler {
    pub fn new() -> Self {
        ListHandler { selected: None }
    }
}
impl EventHandler for ListHandler {
    fn event_id(&self) -> EventId {
        WIDGET_LIST_ITEM_SELECTED
    }
    fn handle_event(&mut self, mut args: EventArgs) {
        let selected = args.data.downcast_ref::<Id>().unwrap();
        if let Some(old_selected) = self.selected {
            if selected != &old_selected {
                args.event_queue.push(EventAddress::SubTree(old_selected),
                                      WIDGET_CHANGE_PROP,
                                      Box::new((Property::Selected, false)));
            }
        }
        self.selected = Some(*selected);
    }
}

pub struct ListItemHandler {
    list_id: Id,
}
impl ListItemHandler {
    pub fn new(list_id: Id) -> Self {
        ListItemHandler { list_id: list_id }
    }
}
impl EventHandler for ListItemHandler {
    fn event_id(&self) -> EventId {
        event::WIDGET_PRESS
    }
    fn handle_event(&mut self, mut args: EventArgs) {
        if let &mut Some(ref drawable) = args.drawable {
            if !drawable.props.contains(&Property::Selected) {
                args.event_queue.push(EventAddress::SubTree(args.widget_id),
                                      WIDGET_CHANGE_PROP,
                                      Box::new((Property::Selected, true)));
                args.event_queue.push(EventAddress::Widget(self.list_id),
                                      WIDGET_LIST_ITEM_SELECTED,
                                      Box::new(args.widget_id));
            }
        }
    }
}
