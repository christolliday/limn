#[allow(unused_imports)]
#[macro_use]
extern crate limn;
#[macro_use]
extern crate limn_layout;

mod util;

use limn::prelude::*;
use limn::widgets::image::ImageBuilder;

fn main() {
    let app = util::init_default("Limn button demo");
    let mut root = WidgetBuilder::new("root");

    let mut image_widget = ImageBuilder::new("rust.png");
    image_widget.layout().add(constraints![
        center(&root),
        bound_by(&root).padding(50.0),
    ]);
    root.add_child(image_widget);

    app.main_loop(root);
}
