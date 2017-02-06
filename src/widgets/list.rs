use linked_hash_map::LinkedHashMap;
use graphics::types::Color;

use widget::{EventArgs, EventHandler, Property, PropSet};
use widgets::primitives::RectStyle;
use widget::style::Value;
use event::{self, EventId, EventAddress, WIDGET_CHANGE_PROP};
use resources::WidgetId;
use color::*;

const WIDGET_LIST_ITEM_SELECTED: EventId = EventId("limn/list_item_selected");

pub struct ListHandler {
    selected: Option<WidgetId>,
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
        let selected = args.data.downcast_ref::<WidgetId>().unwrap();
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
    list_id: WidgetId,
}
impl ListItemHandler {
    pub fn new(list_id: WidgetId) -> Self {
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
