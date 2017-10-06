extern crate limn;
#[macro_use]
extern crate limn_layout;

mod util;

use limn::prelude::*;
use limn::widgets::button::ToggleButtonBuilder;
use limn::widgets::button::PushButtonBuilder;

fn main() {
    let app = util::init_default("Limn button demo");
    let mut root = WidgetBuilder::new("root");

    let mut button = ToggleButtonBuilder::new();
    button.set_text("ON", "OFF");
    button.set_name("button");
    button.layout().add(constraints![
        center(&root),
        bound_by(&root).padding(50.0).strength(WEAK),
    ]);
    let mut image_button = PushButtonBuilder::new();
    let image = limn::draw::image::ImageState::new("rust.png");
    image_button.set_label(image);
    image_button.layout().add(constraints![
        to_right_of(&button),
        center_vertical(&root)
    ]);
    root.add_child(button);
    root.add_child(image_button);

    app.main_loop(root);
}
