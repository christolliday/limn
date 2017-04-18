#[macro_use]
extern crate limn;

mod util;

use limn::widget::{WidgetBuilder, WidgetBuilderCore};
use limn::widgets::list::STYLE_LIST_ITEM;
use limn::drawable::text::{TextDrawable, TextStyleable};
use limn::drawable::rect::RectDrawable;
use limn::util::Dimensions;
use limn::color::*;
use limn::layout::constraint::*;

fn main() {
    let (window, ui) = util::init_default("Limn list demo");
    util::load_default_font();

    let mut root_widget = WidgetBuilder::new();

    let mut scroll_widget = WidgetBuilder::new();
    scroll_widget.contents_scroll();
    layout!(scroll_widget:
        bound_by(&root_widget).padding(50.0),
        dimensions(Dimensions {
            width: 300.0,
            height: 300.0,
        }
    ));

    let mut list_widget = WidgetBuilder::new();
    list_widget.make_vertical_list();
    layout!(list_widget: match_width(&scroll_widget));

    let list_item_widgets = {
        let mut list_item_widgets = Vec::new();
        for _ in 1..15 {
            let text_style = style!(TextStyleable::TextColor: WHITE);
            let text_drawable = TextDrawable::new("hello");
            let text_dims = text_drawable.measure();

            let mut list_item_widget = WidgetBuilder::new();
            list_item_widget
                .set_drawable_with_style(RectDrawable::new(), STYLE_LIST_ITEM.clone())
                .set_debug_name("item")
                .list_item(list_widget.id())
                .enable_hover();
            layout!(list_item_widget: height(text_dims.height));

            let mut list_text_widget = WidgetBuilder::new();
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
    scroll_widget.add_child(list_widget);
    root_widget.add_child(scroll_widget);

    util::set_root_and_loop(window, ui, root_widget);
}
