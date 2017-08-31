#[macro_use]
extern crate limn;
#[macro_use]
extern crate limn_layout;
extern crate lipsum;
extern crate rand;

use lipsum::lipsum;
use rand::Rng;

mod util;

use limn::prelude::*;

use limn::widgets::list::{ListBuilder, STYLE_LIST_ITEM};
use limn::widgets::scroll::ScrollBuilder;
use limn::widgets::text::TextBuilder;
use limn::drawable::text::TextStyleable;
use limn::drawable::rect::RectDrawable;


fn main() {
    let app = util::init_default("Limn list demo");
    let mut root = app.ui.root.clone();

    let mut scroll_widget = ScrollBuilder::new();
    scroll_widget.layout().add(constraints![
        bound_by(&root).padding(50.0),
        size(Size::new(300.0, 300.0)),
    ]);

    let mut list_widget = ListBuilder::new();
    list_widget.layout().add(constraints![shrink(), match_width(&scroll_widget)]);

    let list_data = (0..15).map(|_| {
        let rand = rand::thread_rng().gen_range(2, 6);
        lipsum(rand)
    });
    let text_style = style!(TextStyleable::TextColor: WHITE);
    list_widget.set_contents(list_data, |item, list| {
        let text = (*item).to_owned();
        let style = style!(parent: text_style, TextStyleable::Text: text);
        let mut text_widget = TextBuilder::new_with_style(style);

        let mut item_widget = Widget::new();
        item_widget
            .set_drawable_with_style(RectDrawable::new(), STYLE_LIST_ITEM.clone())
            .set_debug_name("item")
            .list_item(list.widget.clone())
            .enable_hover();

        text_widget.layout().add(align_left(&item_widget));
        item_widget.layout().add(match_width(list));
        item_widget.add_child(text_widget);
        item_widget
    });

    scroll_widget.add_content(list_widget);
    root.add_child(scroll_widget);

    app.main_loop();
}
