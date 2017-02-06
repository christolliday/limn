extern crate limn;

mod util;

use limn::widget::builder::WidgetBuilder;
use limn::widget::layout::{LinearLayout, Orientation};
use limn::widget::{EventHandler, EventArgs, Property, PropsChangeEventHandler};
use limn::widgets::text::{self, TextStyle, TextStyleField};
use limn::widgets::primitives;
use limn::widgets::list::{ListHandler, ListItemHandler};
use limn::widgets::scroll::{ScrollHandler, WidgetScrollHandler};
use limn::widgets::hover::HoverHandler;
use limn::widget::style::Value;
use limn::resources::Id;
use limn::event::{self, EventId, EventAddress};
use limn::util::Dimensions;
use limn::color::*;
use limn::theme::{STYLE_TEXT, STYLE_LIST_ITEM};

fn main() {
    let (window, ui, event_queue) = util::init_default("Limn list demo");
    let font_id = util::load_default_font();

    let mut root_widget = WidgetBuilder::new();

    let mut scroll_widget = WidgetBuilder::new()
        .add_handler(Box::new(ScrollHandler {}))
        .set_scrollable();
    scroll_widget.layout.bound_by(&root_widget, Some(50.0));
    scroll_widget.layout.dimensions(Dimensions {
        width: 300.0,
        height: 300.0,
    });

    let mut list_widget = WidgetBuilder::new()
        .add_handler(Box::new(ListHandler::new()))
        .add_handler(Box::new(WidgetScrollHandler::new()));
    list_widget.layout.match_width(&scroll_widget);


    let list_item_widgets = {
        let mut linear_layout = LinearLayout::new(Orientation::Vertical, &list_widget);
        let mut list_item_widgets = Vec::new();
        for i in 1..15 {
            let text_fields = vec!{
                TextStyleField::text(Value::Single("hello".to_owned())),
                TextStyleField::text_color(Value::Single(WHITE)),
            };
            let text_style = TextStyle::from(text_fields);

            let text_drawable = text::text_drawable(text_style);
            let text_dims = text::measure_dims_no_wrap(&text_drawable);

            let mut list_item_widget = WidgetBuilder::new()
                .set_drawable(primitives::rect_drawable(STYLE_LIST_ITEM.clone()))
                .set_debug_name("item")
                .add_handler(Box::new(HoverHandler {}))
                .add_handler(Box::new(PropsChangeEventHandler {}))
                .add_handler(Box::new(ListItemHandler::new(list_widget.id)));
            list_item_widget.layout.match_width(&list_widget);
            list_item_widget.layout.height(text_dims.height);
            linear_layout.add_widget(&mut list_item_widget);

            let mut list_text_widget = WidgetBuilder::new()
                .set_drawable(text_drawable)
                .set_debug_name("text");
            list_text_widget.layout.center(&list_item_widget);
            list_item_widget.add_child(Box::new(list_text_widget));

            list_item_widgets.push(list_item_widget);
        }
        list_item_widgets
    };

    for list_item_widget in list_item_widgets {
        list_widget.add_child(Box::new(list_item_widget));
    }
    scroll_widget.add_child(Box::new(list_widget));
    root_widget.add_child(Box::new(scroll_widget));

    util::set_root_and_loop(window, ui, root_widget, event_queue, vec!{});
}
