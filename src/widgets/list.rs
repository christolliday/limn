use linked_hash_map::LinkedHashMap;
use graphics::types::Color;

use widget::{EventArgs, EventHandler, Property, PropSet, ChangePropEvent, WidgetNotifyEvent};
use widgets::primitives::{RectDrawable, RectStyle};
use widget::style::StyleSheet;
use event::{self, EventId, EventAddress, Signal};
use resources::Id;
use color::*;

const WIDGET_LIST_ITEM_SELECTED: EventId = EventId("WIDGET_LIST_ITEM_SELECTED");

static COLOR_LIST_ITEM_DEFAULT: Color = [0.3, 0.3, 0.3, 1.0];
static COLOR_LIST_ITEM_HOVER: Color = [0.6, 0.6, 0.6, 1.0];
static COLOR_LIST_ITEM_SELECTED: Color = [0.2, 0.2, 1.0, 1.0];

lazy_static! {
    pub static ref STATE_DEFAULT: PropSet = btreeset!{};
    pub static ref STATE_SELECTED: PropSet = btreeset!{Property::Selected};
    pub static ref STATE_HOVER: PropSet = btreeset!{Property::Hover};
    pub static ref LIST_ITEM_STYLE_DEFAULT: RectStyle = {
        let mut style = LinkedHashMap::new();
        style.insert(STATE_SELECTED.deref().clone(), COLOR_LIST_ITEM_SELECTED);
        style.insert(STATE_HOVER.deref().clone(), COLOR_LIST_ITEM_HOVER);
        RectStyle { background: StyleSheet::new(style, COLOR_LIST_ITEM_DEFAULT) }
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
        let selected = args.event.data::<Id>();
        if let Some(old_selected) = self.selected {
            if selected != &old_selected {
                let event = ChangePropEvent::new(Property::Selected, false);
                args.event_queue.push(EventAddress::SubTree(old_selected), Box::new(event));
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
        if !args.props.contains(&Property::Selected) {
            let event = ChangePropEvent::new(Property::Selected, true);
            args.event_queue.push(EventAddress::SubTree(args.widget_id), Box::new(event));
            let event = WidgetNotifyEvent::new(WIDGET_LIST_ITEM_SELECTED, args.widget_id);
            args.event_queue.push(EventAddress::Widget(self.list_id), Box::new(event));
        }
    }
}

pub struct ListItemPropsHandler {}
impl EventHandler for ListItemPropsHandler {
    fn event_id(&self) -> EventId {
        event::WIDGET_PROPS_CHANGED
    }
    fn handle_event(&mut self, mut args: EventArgs) {
        let selected = args.props.contains(&Property::Selected);
        let hover = args.props.contains(&Property::Hover);
        let color_selected = BLUE;
        let color_hover = [0.6, 0.6, 0.6, 1.0];
        let color_none = [0.3, 0.3, 0.3, 1.0];
        let color =
            if selected { color_selected }
            else if hover { color_hover }
            else { color_none };
        args.state.update(|state: &mut RectDrawable| state.background = color);
    }
}