extern crate limn;
#[macro_use]
extern crate limn_layout;

mod util;

use limn::prelude::*;

use limn::drawable::image::ImageDrawable;

fn main() {
    let app = util::init_default("Limn button demo");

    let mut root_widget = Widget::new();
    let image_drawable = ImageDrawable::new("rust.png");
    let image_size = image_drawable.measure();
    let mut image_widget = Widget::new();
    image_widget.set_drawable(image_drawable);
    layout!(image_widget:
        size(image_size),
        center(&root_widget),
        bound_by(&root_widget).padding(50.0));
    root_widget.add_child(image_widget);

    util::set_root_and_loop(app, root_widget);
}
