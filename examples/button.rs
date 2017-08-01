extern crate limn;
#[macro_use]
extern crate limn_layout;

mod util;

use limn::prelude::*;

use limn::widgets::button::ToggleButtonBuilder;

fn main() {
    let app = util::init_default("Limn button demo");
    util::load_default_font();

    let mut root_widget = WidgetRef::new();
    let mut button = ToggleButtonBuilder::new();
    button.set_text("ON", "OFF");
    button.set_debug_name("button");
    layout!(button:
        center(&root_widget),
        bound_by(&root_widget).padding(50.0));
    root_widget.add_child(button);

    util::set_root_and_loop(app, root_widget);
}
