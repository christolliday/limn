use event::{Target, WidgetEventArgs, WidgetEventHandler};
use widget::{WidgetBuilder, WidgetBuilderCore, BuildWidget};
use widget::property::{Property, PropChange};
use widget::property::states::*;
use layout::{LayoutRef, LayoutVars};
use drawable::rect::RectStyleable;
use resources::WidgetId;
use input::mouse::ClickEvent;
use util::Color;

pub struct ListItemSelected {
    widget: Option<WidgetId>,
}

static COLOR_LIST_ITEM_DEFAULT: Color = [0.3, 0.3, 0.3, 1.0];
static COLOR_LIST_ITEM_MOUSEOVER: Color = [0.6, 0.6, 0.6, 1.0];
static COLOR_LIST_ITEM_SELECTED: Color = [0.2, 0.2, 1.0, 1.0];

lazy_static! {
    pub static ref STYLE_LIST_ITEM: Vec<RectStyleable> = {
        style!(RectStyleable::BackgroundColor: selector!(COLOR_LIST_ITEM_DEFAULT,
            SELECTED: COLOR_LIST_ITEM_SELECTED,
            MOUSEOVER: COLOR_LIST_ITEM_MOUSEOVER))
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
impl WidgetEventHandler<ListItemSelected> for ListHandler {
    fn handle(&mut self, event: &ListItemSelected, _: WidgetEventArgs) {
        let selected = event.widget;
        if selected != self.selected {
            if let Some(old_selected) = self.selected {
                event!(Target::SubTree(old_selected), PropChange::Remove(Property::Selected));
            }
        }
        self.selected = selected;
    }
}

fn list_handle_deselect(_: &ClickEvent, args: WidgetEventArgs) {
    event!(Target::Widget(args.widget.id), ListItemSelected { widget: None });
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
            event!(Target::SubTree(args.widget.id), PropChange::Add(Property::Selected));
            let event = ListItemSelected { widget: Some(args.widget.id) };
            event!(Target::Widget(self.list_id), event);
            *args.handled = true;
        }
    }
}

pub struct ListBuilder {
    pub widget: WidgetBuilder,
}
widget_builder!(ListBuilder);

impl ListBuilder {
    pub fn new() -> Self {
        let mut widget = WidgetBuilder::new();
        widget.add_handler(ListHandler::new())
              .add_handler_fn(list_handle_deselect)
              .vbox();
        ListBuilder {
            widget: widget,
        }
    }
    pub fn on_item_selected<F>(&mut self, on_item_selected: F) -> &mut Self
        where F: Fn(Option<WidgetId>, WidgetEventArgs) + 'static
    {
        self.widget.add_handler_fn(move |event: &ListItemSelected, args| {
            on_item_selected(event.widget, args);
        });
        self
    }
}

impl WidgetBuilder {
    pub fn list_item(&mut self, list_id: WidgetId) -> &mut Self {
        self.add_handler(ListItemHandler::new(list_id))
    }
}
