extern crate limn;
#[macro_use]
extern crate limn_layout;

mod util;

use limn::prelude::*;
use limn::widgets::button::ToggleButtonBuilder;

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
    root.add_child(button);

    app.main_loop(root);
}
