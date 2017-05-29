#[macro_use]
extern crate limn;
#[macro_use]
extern crate limn_layout;

mod util;

use limn::widget::WidgetBuilder;
use limn::widget::WidgetBuilderCore;
use limn::widgets::button::ToggleButtonBuilder;
use limn::layout::constraint::*;

fn main() {
    let (window, ui) = util::init_default("Limn button demo");
    util::load_default_font();

    let mut root_widget = WidgetBuilder::new();
    let mut button = ToggleButtonBuilder::new();
    button.set_text("ON", "OFF");
    button.set_debug_name("button");
    layout!(button:
        center(&root_widget),
        bound_by(&root_widget).padding(50.0));
    root_widget.add_child(button);

    util::set_root_and_loop(window, ui, root_widget);
}
