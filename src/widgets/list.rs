use linked_hash_map::LinkedHashMap;

use widget::{EventArgs, EventHandler};
use widget::property::Property;
use widget::property::states::*;
use widget::style::Value;
use widgets::primitives::RectStyleField;
use event::{EventId, EventAddress};
use event::events::*;
use event::id::*;
use resources::WidgetId;
use util::Color;

pub struct WidgetListItemSelected(WidgetId);

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
impl EventHandler<WidgetListItemSelected> for ListHandler {
    fn handle(&mut self, mut args: EventArgs<WidgetListItemSelected>) {
        let selected: WidgetId = args.event.0;
        if let Some(old_selected) = self.selected {
            if selected != old_selected {
                args.event_queue.change_prop(old_selected, Property::Selected, false);
            }
        }
        self.selected = Some(selected);
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
impl EventHandler<WidgetMouseButton> for ListItemHandler {
    fn handle(&mut self, mut args: EventArgs<WidgetMouseButton>) {
        if let &mut Some(ref drawable) = args.drawable {
            if !drawable.props.contains(&Property::Selected) {
                args.event_queue.change_prop(args.widget_id, Property::Selected, true);
                args.event_queue.push(EventAddress::Widget(self.list_id),
                                      NONE,
                                      WidgetListItemSelected(args.widget_id));
            }
        }
    }
}
