extern crate limn;
#[macro_use]
extern crate limn_layout;

mod util;

use limn::widgets::image::ImageBuilder;

fn main() {
    let app = util::init_default("Limn button demo");
    let mut root = app.ui.root.clone();

    let mut image_widget = ImageBuilder::new("rust.png");
    layout!(image_widget:
        center(&root),
        bound_by(&root).padding(50.0));
    root.add_child(image_widget);

    app.main_loop();
}
