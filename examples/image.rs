extern crate limn;
#[macro_use]
extern crate limn_layout;

mod util;

use limn::prelude::*;
use limn::widgets::image::ImageBuilder;

fn main() {
    let app = util::init_default("Limn button demo");

    let mut root_widget = Widget::new();
    let mut image_widget = ImageBuilder::new("rust.png");
    layout!(image_widget:
        center(&root_widget),
        bound_by(&root_widget).padding(50.0));
    root_widget.add_child(image_widget);

    util::set_root_and_loop(app, root_widget);
}
