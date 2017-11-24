#[macro_use]
extern crate limn;
extern crate lipsum;
extern crate rand;

use lipsum::lipsum;
use rand::Rng;

mod util;

use limn::prelude::*;

use limn::widgets::list::{self, ListBuilder};
use limn::widgets::scroll::ScrollBuilder;

fn main() {
    let window_builder = glutin::WindowBuilder::new()
        .with_title("Limn list demo")
        .with_min_dimensions(100, 300);
    let app = util::init(window_builder);
    let mut root = WidgetBuilder::new("root");

    let mut scroll_widget = ScrollBuilder::new();
    scroll_widget.layout().add(constraints![
        match_layout(&root).padding(50.0),
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
