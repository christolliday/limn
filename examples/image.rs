#[macro_use]
extern crate limn;
#[macro_use]
extern crate limn_layout;

mod util;

use limn::prelude::*;

use limn::drawable::image::ImageDrawable;

fn main() {
    let (mut window, ui) = util::init_default("Limn button demo");
    let image_id = util::load_default_image(&mut window);

    let mut root_widget = WidgetBuilder::new();
    let image_drawable = ImageDrawable::new(image_id);
    let image_size = image_drawable.measure();
    let mut image_widget = WidgetBuilder::new();
    image_widget.set_drawable(image_drawable);
    layout!(image_widget:
        size(image_size),
        center(&root_widget),
        bound_by(&root_widget).padding(50.0));
    root_widget.add_child(image_widget);

    util::set_root_and_loop(window, ui, root_widget);
}
