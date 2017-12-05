#[allow(unused_imports)]
#[macro_use]
extern crate limn;

mod util;

use limn::prelude::*;
use limn::widgets::image::Image;

fn main() {
    let window_builder = glutin::WindowBuilder::new()
        .with_title("Limn image demo")
        .with_min_dimensions(100, 100);
    let app = util::init(window_builder);
    resources().image_loader.load_image("rust", include_bytes!("../assets/images/rust.png").to_vec());

    let mut root = Widget::new("root");

    let mut image_widget = Widget::from_modifier(Image::new(ImageSource::bundled("rust")));
    image_widget.layout().add(constraints![
        center(&root),
        bound_by(&root).padding(50.0),
    ]);
    root.add_child(image_widget);

    app.main_loop(root);
}
