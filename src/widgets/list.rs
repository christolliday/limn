use event::{WidgetEventArgs, WidgetEventHandler};
use widget::Widget;
use widget::property::{Property, PropChange};
use widget::property::states::*;
use drawable::rect::RectStyleable;
use input::mouse::ClickEvent;
use color::*;

pub struct ListItemSelected {
    widget: Option<Widget>,
}

static COLOR_LIST_ITEM_DEFAULT: Color = GRAY_30;
static COLOR_LIST_ITEM_MOUSEOVER: Color = GRAY_60;
static COLOR_LIST_ITEM_SELECTED: Color = BLUE_HIGHLIGHT;

lazy_static! {
    pub static ref STYLE_LIST_ITEM: Vec<RectStyleable> = {
        style!(RectStyleable::BackgroundColor: selector!(COLOR_LIST_ITEM_DEFAULT,
            SELECTED: COLOR_LIST_ITEM_SELECTED,
            MOUSEOVER: COLOR_LIST_ITEM_MOUSEOVER))
    };
}

pub struct ListHandler {
    selected: Option<Widget>,
}
impl ListHandler {
    pub fn new() -> Self {
        ListHandler { selected: None }
    }
}
impl WidgetEventHandler<ListItemSelected> for ListHandler {
    fn handle(&mut self, event: &ListItemSelected, _: WidgetEventArgs) {
        let selected = event.widget.clone();
        if selected != self.selected {
            if let Some(ref old_selected) = self.selected {
                old_selected.event_subtree(PropChange::Remove(Property::Selected));
            }
        }
        self.selected = selected;
    }
}

fn list_handle_deselect(_: &ClickEvent, args: WidgetEventArgs) {
    args.widget.event(ListItemSelected { widget: None });
}

pub struct ListItemHandler {
    list_id: Widget,
}
impl ListItemHandler {
    pub fn new(list_id: Widget) -> Self {
        ListItemHandler { list_id: list_id }
    }
}
impl WidgetEventHandler<ClickEvent> for ListItemHandler {
    fn handle(&mut self, _: &ClickEvent, mut args: WidgetEventArgs) {
        if !args.widget.props().contains(&Property::Selected) {
            args.widget.event_subtree(PropChange::Add(Property::Selected));
            let event = ListItemSelected { widget: Some(args.widget) };
            self.list_id.event(event);
            *args.handled = true;
        }
    }
}

pub struct ListBuilder {
    pub widget: Widget,
}
widget_wrapper!(ListBuilder);

impl ListBuilder {
    pub fn new() -> Self {
        let mut widget = Widget::new();
        widget.add_handler(ListHandler::new())
              .add_handler_fn(list_handle_deselect)
              .vbox();
        ListBuilder {
            widget: widget,
        }
    }
    pub fn on_item_selected<F>(&mut self, on_item_selected: F) -> &mut Self
        where F: Fn(Option<Widget>, WidgetEventArgs) + 'static
    {
        self.widget.add_handler_fn(move |event: &ListItemSelected, args| {
            on_item_selected(event.widget.clone(), args);
        });
        self
    }
}

impl Widget {
    pub fn list_item(&mut self, list_id: Widget) -> &mut Self {
        self.add_handler(ListItemHandler::new(list_id))
    }
}
