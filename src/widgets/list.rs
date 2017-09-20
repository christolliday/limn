use event::{WidgetEventArgs, WidgetEventHandler};
use widget::{WidgetBuilder, WidgetRef};
use widget::property::{Property, PropChange};
use widget::property::states::*;
use widgets::text::TextBuilder;
use draw::rect::{RectState, RectStyle};
use draw::text::TextStyle;
use input::mouse::ClickEvent;
use layout::constraint::*;
use color::*;

pub struct ListItemSelected {
    widget: Option<WidgetRef>,
}

static COLOR_LIST_ITEM_DEFAULT: Color = GRAY_30;
static COLOR_LIST_ITEM_MOUSEOVER: Color = GRAY_60;
static COLOR_LIST_ITEM_SELECTED: Color = BLUE_HIGHLIGHT;

lazy_static! {
    pub static ref STYLE_LIST_ITEM: Vec<RectStyle> = {
        style!(RectStyle::BackgroundColor: selector!(COLOR_LIST_ITEM_DEFAULT,
            SELECTED: COLOR_LIST_ITEM_SELECTED,
            MOUSEOVER: COLOR_LIST_ITEM_MOUSEOVER))
    };
    pub static ref STYLE_LIST_TEXT: Vec<TextStyle> = {
        style!(TextStyle::TextColor: WHITE)
    };
}

pub struct ListHandler {
    selected: Option<WidgetRef>,
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
    list_id: WidgetRef,
}
impl ListItemHandler {
    pub fn new(list_id: WidgetRef) -> Self {
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
    pub widget: WidgetBuilder,
}
widget_wrapper!(ListBuilder);

impl ListBuilder {
    pub fn new() -> Self {
        let mut widget = WidgetBuilder::new("list");
        widget.add_handler(ListHandler::new())
              .add_handler_fn(list_handle_deselect)
              .vbox();
        ListBuilder {
            widget: widget,
        }
    }
    pub fn on_item_selected<F>(&mut self, on_item_selected: F) -> &mut Self
        where F: Fn(Option<WidgetRef>, WidgetEventArgs) + 'static
    {
        self.widget.add_handler_fn(move |event: &ListItemSelected, args| {
            on_item_selected(event.widget.clone(), args);
        });
        self
    }
    pub fn set_contents<C, I, F>(&mut self, contents: C, build: F)
        where C: Iterator<Item=I>,
              F: Fn(I, &mut ListBuilder) -> WidgetBuilder,
    {
        for item in contents {
            let mut widget = build(item, self);
            widget
                .set_name("list_item")
                .list_item(&self.widget.widget_ref());
            self.widget.add_child(widget);
        }
    }
}

impl WidgetBuilder {
    pub fn list_item(&mut self, parent_list: &WidgetRef) -> &mut Self {
        self.add_handler(ListItemHandler::new(parent_list.clone()))
    }
}

pub fn default_text_adapter(item: String, list: &mut ListBuilder) -> WidgetBuilder {
    let text = (*item).to_owned();
    let style = style!(parent: STYLE_LIST_TEXT, TextStyle::Text: text);
    let mut text_widget = TextBuilder::new_with_style(style);

    let mut item_widget = WidgetBuilder::new("list_item");
    item_widget
        .set_draw_state_with_style(RectState::new(), STYLE_LIST_ITEM.clone())
        .enable_hover();

    text_widget.layout().add(align_left(&item_widget));
    item_widget.layout().add(match_width(list));
    item_widget.add_child(text_widget);
    item_widget
}
