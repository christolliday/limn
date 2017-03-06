extern crate limn;

mod util;

use limn::widget::builder::WidgetBuilder;
use limn::widget::layout::{LinearLayout, Orientation};
use limn::widgets::text::{TextDrawable, TextStyleField};
use limn::widgets::primitives::RectDrawable;
use limn::widgets::list::{ListHandler, ListItemHandler, STYLE_LIST_ITEM};
use limn::widget::style::Value;
use limn::util::Dimensions;
use limn::color::*;

fn main() {
    let (window, ui) = util::init_default("Limn list demo");
    util::load_default_font();

    let mut root_widget = WidgetBuilder::new();

    let mut scroll_widget = WidgetBuilder::new().contents_scroll();
    scroll_widget.layout.bound_by(&root_widget, Some(50.0));
    scroll_widget.layout.dimensions(Dimensions {
        width: 300.0,
        height: 300.0,
    });

    let mut list_widget = WidgetBuilder::new()
        .add_handler(ListHandler::new())
        .scrollable();
    list_widget.layout.match_width(&scroll_widget);


    let list_item_widgets = {
        let mut linear_layout = LinearLayout::new(Orientation::Vertical, &mut list_widget);
        let mut list_item_widgets = Vec::new();
        for _ in 1..15 {
            let text_style = vec![TextStyleField::Text(Value::Single("hello".to_owned())),
                                  TextStyleField::TextColor(Value::Single(WHITE))];

            let text_drawable = TextDrawable::new();
            let text_dims = text_drawable.measure();

            let mut list_item_widget = WidgetBuilder::new()
                .set_drawable_with_style(RectDrawable::new(), STYLE_LIST_ITEM.clone())
                .set_debug_name("item")
                .props_may_change()
                .enable_hover()
                .add_handler(ListItemHandler::new(list_widget.id));
            list_item_widget.layout.match_width(&list_widget);
            list_item_widget.layout.height(text_dims.height);
            linear_layout.add_widget(&mut list_item_widget);

            let mut list_text_widget = WidgetBuilder::new()
                .set_drawable_with_style(text_drawable, text_style)
                .set_debug_name("text");
            list_text_widget.layout.center(&list_item_widget);
            list_item_widget.add_child(list_text_widget);

            list_item_widgets.push(list_item_widget);
        }
        list_item_widgets
    };

    for list_item_widget in list_item_widgets {
        list_widget.add_child(list_item_widget);
    }
    scroll_widget.add_child(list_widget);
    root_widget.add_child(scroll_widget);

    util::set_root_and_loop(window, ui, root_widget);
}
