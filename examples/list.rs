#![allow(unused_imports)]

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

use limn::widgets::list::{self, ListBuilder};
use limn::widgets::scroll::ScrollBuilder;


fn main() {
    let app = util::init_default("Limn list demo");
    let mut root = WidgetBuilder::new("root");

    let mut scroll_widget = ScrollBuilder::new();
    scroll_widget.layout().add(constraints![
        match_layout(&root).padding(50.0),
        min_size(Size::new(300.0, 300.0)),
    ]);

    let mut list_widget = ListBuilder::new();
    list_widget.layout().add(constraints![shrink(), match_width(&scroll_widget)]);

    let list_data = (0..15).map(|_| {
        let rand = rand::thread_rng().gen_range(1, 6);
        lipsum(rand)
    });
    list_widget.set_contents(list_data, list::default_text_adapter);

    scroll_widget.add_content(list_widget);
    root.add_child(scroll_widget);

    app.main_loop(root);
}
