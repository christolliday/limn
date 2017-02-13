use linked_hash_map::LinkedHashMap;

use widget::{EventArgs, EventHandler};
use widget::property::Property;
use widget::property::states::*;
use widget::style::Value;
use widgets::primitives::RectStyleField;
use event::{EventId, EventAddress};
use event::id::*;
use resources::WidgetId;
use util::Color;

const WIDGET_LIST_ITEM_SELECTED: EventId = EventId("limn/list_item_selected");

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
}

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
                args.event_queue.change_prop(old_selected, Property::Selected, false);
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
        WIDGET_MOUSE_BUTTON
    }
    fn handle_event(&mut self, mut args: EventArgs) {
        if let &mut Some(ref drawable) = args.drawable {
            if !drawable.props.contains(&Property::Selected) {
                args.event_queue.change_prop(args.widget_id, Property::Selected, true);
                args.event_queue.push(EventAddress::Widget(self.list_id),
                                      WIDGET_LIST_ITEM_SELECTED,
                                      args.widget_id);
            }
        }
    }
}
