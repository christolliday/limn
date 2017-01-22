extern crate limn;

mod util;

use limn::widget::builder::WidgetBuilder;
use limn::widget::image::ImageDrawable;
use limn::widget;

fn main() {
    let (mut window, ui) = util::init_default("Limn button demo");
    let image_id = util::load_default_image(&mut window);

    let mut root_widget = WidgetBuilder::new();
    let mut image_widget = WidgetBuilder::new();
    let image_drawable = ImageDrawable::new(image_id);
    let image_dims = image_drawable.measure_image();
    image_widget.set_drawable(widget::image::draw_image, Box::new(image_drawable));
    image_widget.layout.dimensions(image_dims);
    image_widget.layout.center(&root_widget.layout);
    image_widget.layout.pad(50.0, &root_widget.layout);
    root_widget.add_child(Box::new(image_widget));

    util::set_root_and_loop(window, ui, root_widget);
}
