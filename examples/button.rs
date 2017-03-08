extern crate limn;

mod util;

use limn::widget::WidgetBuilder;
use limn::widgets::button::ToggleButtonBuilder;

fn main() {
    let (window, ui) = util::init_default("Limn button demo");
    util::load_default_font();

    let mut root_widget = WidgetBuilder::new();
    let mut button = ToggleButtonBuilder::new()
        .set_text("ON", "OFF")
        .widget;
    button.layout.center(&root_widget);
    button.layout.bound_by(&root_widget, Some(50.0));
    root_widget.add_child(button);

    util::set_root_and_loop(window, ui, root_widget);
}
