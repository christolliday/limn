use prelude::*;
use draw::prelude::*;
use widgets::text::StaticTextStyle;

pub struct ListItemSelected {
    pub widget: Option<Widget>,
}

#[derive(Debug, Copy, Clone)]
pub struct ItemSelected;

#[derive(Default)]
pub struct ListHandler {
    selected: Option<Widget>,
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
    list_widget: Widget,
}

impl ListItemHandler {
    pub fn new(list_widget: Widget) -> Self {
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
    fn apply(&self, widget: &mut Widget) {
        widget
            .add_handler(ListHandler::default())
            .add_handler(|_: &ClickEvent, args: EventArgs| {
                args.widget.event(ListItemSelected { widget: None });
            })
            .linear_layout(self.layout_settings);
    }
}

pub fn add_contents_to_list<C, I, F>(list: &mut Widget, contents: C, build: F)
    where C: Iterator<Item=I>,
          F: Fn(I, &mut Widget) -> Widget,
{
    for item in contents {
        let mut widget = build(item, list);
        widget
            .set_name("list_item")
            .add_handler(ListItemHandler::new(list.clone()));
        list.add_child(widget);
    }
}

pub fn default_text_adapter(text: String, list: &mut Widget) -> Widget {
    let mut text_widget = Widget::new("list_item_text");
    text_widget.set_draw_style(DrawStyle::from_class::<TextStyle>("list_item_text"));
    StaticTextStyle::from_text(&text).component().apply(&mut text_widget);

    let mut item_widget = Widget::new("list_item_rect");
    item_widget.set_draw_style(DrawStyle::from_class::<RectStyle>("list_item_rect"))
        .enable_hover();

    text_widget.layout().add(align_left(&item_widget));
    item_widget.layout().add(match_width(list));
    item_widget.add_child(text_widget);
    item_widget
}
