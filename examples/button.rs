#[allow(unused_imports)]
#[macro_use]
extern crate limn;

mod util;

use limn::prelude::*;
use limn::widgets::button::ToggleButtonStyle;

fn main() {
    let window_builder = glutin::WindowBuilder::new()
        .with_title("Limn button demo")
        .with_min_dimensions(100, 100);
    let app = util::init(window_builder);
    let mut root = Widget::new("root");

    let mut button = ToggleButtonStyle::default();
    button.toggle_text("ON", "OFF");
    let mut button = Widget::from_modifier_style(button);
    button.layout().add(constraints![
        center(&root),
        bound_by(&root).padding(50.0).strength(WEAK),
    ]);
    root.layout().add(shrink());
    root.add_child(button);

    app.main_loop(root);
}
