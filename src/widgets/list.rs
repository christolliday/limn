use std::any::TypeId;

use event::{EventArgs, EventHandler};
use widget::{WidgetBuilder, WidgetRef};
use widget::property::Property;
use widgets::text::StaticTextStyle;
use draw::rect::RectStyle;
use draw::text::TextStyle;
use input::mouse::ClickEvent;
use layout::constraint::*;
use layout::linear_layout::{LinearLayoutSettings, Orientation, ItemAlignment};
use style::{WidgetModifier, ComponentStyle};

pub struct ListItemSelected {
    pub widget: Option<WidgetRef>,
}

#[derive(Debug, Copy, Clone)]
pub struct ItemSelected;

#[derive(Default)]
pub struct ListHandler {
    selected: Option<WidgetRef>,
}

impl EventHandler<ListItemSelected> for ListHandler {
    fn handle(&mut self, event: &ListItemSelected, _: EventArgs) {
        let selected = event.widget.clone();
        if selected != self.selected {
            if let Some(ref mut old_selected) = self.selected {
                old_selected.remove_prop(Property::Selected);
            }
        }
        self.selected = selected;
    }
}

pub struct ListItemHandler {
    list_widget: WidgetRef,
}

impl ListItemHandler {
    pub fn new(list_widget: WidgetRef) -> Self {
        ListItemHandler { list_widget: list_widget }
    }
}

impl EventHandler<ClickEvent> for ListItemHandler {
    fn handle(&mut self, _: &ClickEvent, mut args: EventArgs) {
        if !args.widget.props().contains(&Property::Selected) {
            args.widget.add_prop(Property::Selected);
            let event = ListItemSelected { widget: Some(args.widget.clone()) };
            self.list_widget.event(event);
            args.widget.event(ItemSelected);
            *args.handled = true;
        }
    }
}

component_style!{pub struct List<name="list", style=ListStyle> {
    layout_settings: LinearLayoutSettings = {
        let mut layout_settings = LinearLayoutSettings::new(Orientation::Vertical);
        layout_settings.item_align = ItemAlignment::Fill;
        layout_settings
    },
}}

impl WidgetModifier for List {
    fn apply(&self, widget: &mut WidgetBuilder) {
        widget
            .add_handler(ListHandler::default())
            .add_handler(|_: &ClickEvent, args: EventArgs| {
                args.widget.event(ListItemSelected { widget: None });
            })
            .linear_layout(self.layout_settings);
    }
}

impl WidgetBuilder {
    pub fn list_item(&mut self, parent_list: &WidgetRef) -> &mut Self {
        self.add_handler(ListItemHandler::new(parent_list.clone()))
    }

    pub fn on_item_selected<F>(&mut self, on_item_selected: F) -> &mut Self
        where F: Fn(EventArgs) + 'static
    {
        self.add_handler(move |_: &ItemSelected, args: EventArgs| {
            on_item_selected(args);
        });
        self
    }

    pub fn set_contents<C, I, F>(&mut self, contents: C, build: F)
        where C: Iterator<Item=I>,
              F: Fn(I, &mut WidgetBuilder) -> WidgetBuilder,
    {
        for item in contents {
            let mut widget = build(item, self);
            widget
                .set_name("list_item")
                .list_item(&self.widget_ref());
            self.widget.add_child(widget);
        }
    }
}

pub fn default_text_adapter(text: String, list: &mut WidgetBuilder) -> WidgetBuilder {
    let mut text_widget = WidgetBuilder::new("list_item_text");
    text_widget.set_style_class(TypeId::of::<TextStyle>(), "list_item_text");
    StaticTextStyle::from_text(&text).component().apply(&mut text_widget);

    let mut item_widget = WidgetBuilder::new("list_item_rect");
    item_widget
        .set_style_class(TypeId::of::<RectStyle>(), "list_item_rect")
        .set_draw_style(RectStyle::default())
        .enable_hover();

    text_widget.layout().add(align_left(&item_widget));
    item_widget.layout().add(match_width(list));
    item_widget.add_child(text_widget);
    item_widget
}
