extern crate limn;

mod util;

use limn::widget::WidgetBuilder;
use limn::widget::WidgetBuilderCore;
use limn::widgets::button::ToggleButtonBuilder;

fn main() {
    let (window, ui) = util::init_default("Limn button demo");
    util::load_default_font();

    let mut root_widget = WidgetBuilder::new();
    let mut button = ToggleButtonBuilder::new();
    button.set_text("ON", "OFF");
    button.set_debug_name("button");
    button.layout().center(&root_widget.layout().vars);
    button.layout().bound_by(&root_widget.layout().vars).padding(50.0);
    root_widget.add_child(button.widget);

    util::set_root_and_loop(window, ui, root_widget);
}
