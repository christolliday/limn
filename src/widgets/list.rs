use std::any::TypeId;

use event::{EventArgs, EventHandler};
use widget::{WidgetBuilder, WidgetRef};
use widget::property::Property;
use widgets::text::StaticTextStyle;
use draw::rect::RectComponentStyle;
use draw::text::TextComponentStyle;
use input::mouse::ClickEvent;
use layout::constraint::*;
use layout::linear_layout::{LinearLayoutSettings, Orientation, ItemAlignment};
use style::{ComponentStyle, WidgetModifier};

pub struct ListItemSelected {
    pub widget: Option<WidgetRef>,
}

#[derive(Debug, Copy, Clone)]
pub struct ItemSelected;


#[derive(Default)]
pub struct ListHandler {
    selected: Option<WidgetRef>,
}

impl ListHandler {
    pub fn new() -> Self {
        Self::default()
    }
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

#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn list_handle_deselect(_: &ClickEvent, args: EventArgs) {
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

impl EventHandler<ClickEvent> for ListItemHandler {
    fn handle(&mut self, _: &ClickEvent, mut args: EventArgs) {
        if !args.widget.props().contains(&Property::Selected) {
            args.widget.add_prop(Property::Selected);
            let event = ListItemSelected { widget: Some(args.widget) };
            self.list_id.event(event);
            *args.handled = true;
        }
    }
}

component_style!{pub struct ListBuilder<name="list", style=ListBuilderStyle> {
    layout: LinearLayoutSettings = {
        let mut layout_settings = LinearLayoutSettings::new(Orientation::Vertical);
        layout_settings.item_align = ItemAlignment::Fill;
        layout_settings
    },
}}

impl WidgetModifier for ListBuilder {
    fn apply(&self, widget: &mut WidgetBuilder) {
        let mut layout_settings = LinearLayoutSettings::new(Orientation::Vertical);
        layout_settings.item_align = ItemAlignment::Fill;

        widget.add_handler(ListHandler::new())
              .add_handler(&list_handle_deselect)
              .linear_layout(layout_settings);
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
    /// Set the contents of the list
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
    let mut style = StaticTextStyle::default();
    style.text(&text);
    let mut text_widget = WidgetBuilder::new("list_item_text");
    text_widget.set_style_class(TypeId::of::<TextComponentStyle>(), "list_item_text");
    style.component().apply(&mut text_widget);

    let mut item_widget = WidgetBuilder::new("list_item_rect");
    item_widget
        .set_style_class(TypeId::of::<RectComponentStyle>(), "list_item_rect")
        .set_draw_style(RectComponentStyle::default())
        .enable_hover();

    text_widget.layout().add(align_left(&item_widget));
    item_widget.layout().add(match_width(list));
    item_widget.add_child(text_widget);
    item_widget
}
