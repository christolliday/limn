extern crate limn;

mod util;

use limn::widget::WidgetBuilder;
use limn::widget::WidgetBuilderCore;
use limn::widget::style::Value;
use limn::widgets::list::STYLE_LIST_ITEM;
use limn::drawable::text::{TextDrawable, TextStyleField};
use limn::drawable::rect::RectDrawable;
use limn::util::Dimensions;
use limn::color::*;

fn main() {
    let (window, ui) = util::init_default("Limn list demo");
    util::load_default_font();

    let mut root_widget = WidgetBuilder::new();

    let mut scroll_widget = WidgetBuilder::new();
    scroll_widget.contents_scroll();
    scroll_widget.layout().bound_by(&root_widget.layout()).padding(50.0);
    scroll_widget.layout().dimensions(Dimensions {
        width: 300.0,
        height: 300.0,
    });

    let mut list_widget = WidgetBuilder::new();
    list_widget.make_vertical_list();
    list_widget.layout().match_width(&scroll_widget.layout());

    let list_item_widgets = {
        let mut list_item_widgets = Vec::new();
        for _ in 1..15 {
            let text_style = vec![TextStyleField::TextColor(Value::Single(WHITE))];
            let text_drawable = TextDrawable::new("hello");
            let text_dims = text_drawable.measure();

            let mut list_item_widget = WidgetBuilder::new();
            list_item_widget
                .set_drawable_with_style(RectDrawable::new(), STYLE_LIST_ITEM.clone())
                .set_debug_name("item")
                .list_item(list_widget.id())
                .enable_hover();
            list_item_widget.layout().height(text_dims.height);

            let mut list_text_widget = WidgetBuilder::new();
            list_text_widget
                .set_drawable_with_style(text_drawable, text_style)
                .set_debug_name("text");
            list_text_widget.layout().center(&list_item_widget.layout());
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
