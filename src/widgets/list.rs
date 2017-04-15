use event::{Target, WidgetEventArgs, WidgetEventHandler};
use widget::{WidgetBuilder, WidgetBuilderCore};
use widget::property::{Property, PropChange};
use widget::property::states::*;
use drawable::rect::RectStyleable;
use resources::WidgetId;
use input::mouse::ClickEvent;
use util::Color;

pub struct WidgetListItemSelected {
    widget: Option<WidgetId>,
}

static COLOR_LIST_ITEM_DEFAULT: Color = [0.3, 0.3, 0.3, 1.0];
static COLOR_LIST_ITEM_HOVER: Color = [0.6, 0.6, 0.6, 1.0];
static COLOR_LIST_ITEM_SELECTED: Color = [0.2, 0.2, 1.0, 1.0];

lazy_static! {
    pub static ref STYLE_LIST_ITEM: Vec<RectStyleable> = {
        style!(RectStyleable::BackgroundColor: selector!(COLOR_LIST_ITEM_DEFAULT,
            SELECTED: COLOR_LIST_ITEM_SELECTED,
            HOVER: COLOR_LIST_ITEM_HOVER))
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
impl WidgetEventHandler<WidgetListItemSelected> for ListHandler {
    fn handle(&mut self, event: &WidgetListItemSelected, mut args: WidgetEventArgs) {
        let selected = event.widget;
        if selected != self.selected {
            if let Some(old_selected) = self.selected {
                args.queue.push(Target::SubTree(old_selected), PropChange::Remove(Property::Selected));
            }
        }
        self.selected = selected;
    }
}

fn list_handle_deselect(_: &ClickEvent, mut args: WidgetEventArgs) {
    args.queue.push(Target::Widget(args.widget.id), WidgetListItemSelected { widget: None });
}

pub struct ListItemHandler {
    list_id: WidgetId,
}
impl ListItemHandler {
    pub fn new(list_id: WidgetId) -> Self {
        ListItemHandler { list_id: list_id }
    }
}
impl WidgetEventHandler<ClickEvent> for ListItemHandler {
    fn handle(&mut self, _: &ClickEvent, mut args: WidgetEventArgs) {
       if !args.widget.props.contains(&Property::Selected) {
            args.queue.push(Target::SubTree(args.widget.id), PropChange::Add(Property::Selected));
            let event = WidgetListItemSelected { widget: Some(args.widget.id) };
            args.queue.push(Target::Widget(self.list_id), event);
            *args.handled = true;
        }
    }
}

impl WidgetBuilder {
    pub fn make_vertical_list(&mut self) -> &mut Self {
        self.add_handler(ListHandler::new())
            .add_handler_fn(list_handle_deselect)
            .vbox()
            .make_scrollable()
    }

    pub fn list_item(&mut self, list_id: WidgetId) -> &mut Self {
        self.add_handler(ListItemHandler::new(list_id))
    }
}