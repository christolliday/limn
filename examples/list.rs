#[macro_use]
extern crate limn;
#[macro_use]
extern crate limn_layout;

mod util;

use limn::prelude::*;

use limn::widgets::list::{ListBuilder, STYLE_LIST_ITEM};
use limn::widgets::scroll::ScrollBuilder;
use limn::drawable::text::{TextDrawable, TextStyleable};
use limn::drawable::rect::RectDrawable;

fn main() {
    let app = util::init_default("Limn list demo");
    util::load_default_font();

    let mut root_widget = Widget::new();

    let mut scroll_widget = ScrollBuilder::new();
    layout!(scroll_widget:
        bound_by(&root_widget).padding(50.0),
        size(Size::new(300.0, 300.0)),
     );

    let mut list_widget = ListBuilder::new();
    layout!(list_widget: match_width(&scroll_widget));

    let list_item_widgets = {
        let mut list_item_widgets = Vec::new();
        for _ in 1..15 {
            let text_style = style!(TextStyleable::TextColor: WHITE);
            let text_drawable = TextDrawable::new("hello");
            let text_size = text_drawable.measure();

            let mut list_item_widget = Widget::new();
            list_item_widget
                .set_drawable_with_style(RectDrawable::new(), STYLE_LIST_ITEM.clone())
                .set_debug_name("item")
                .list_item(list_widget.widget.clone())
                .enable_hover();
            layout!(list_item_widget: height(text_size.height));

            let mut list_text_widget = Widget::new();
            list_text_widget
                .set_drawable_with_style(text_drawable, text_style)
                .set_debug_name("text");
            layout!(list_text_widget: center(&list_item_widget));
            list_item_widget.add_child(list_text_widget);

            list_item_widgets.push(list_item_widget);
        }
        list_item_widgets
    };

    for list_item_widget in list_item_widgets {
        list_widget.add_child(list_item_widget);
    }
    scroll_widget.add_content(list_widget);
    root_widget.add_child(scroll_widget);

    util::set_root_and_loop(app, root_widget);
}
