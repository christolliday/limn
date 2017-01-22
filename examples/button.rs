extern crate limn;

mod util;

use limn::widget::builder::WidgetBuilder;
use limn::widgets::button::ToggleButtonBuilder;

fn main() {
    let (window, ui) = util::init_default("Limn button demo");
    let font_id = util::load_default_font();

    let mut root_widget = WidgetBuilder::new();
    let mut button = ToggleButtonBuilder::new();
    button.set_text("ON", "OFF", font_id);
    button.widget.layout.center(&root_widget.layout);
    button.widget.layout.pad(50.0, &root_widget.layout);
    root_widget.add_child(Box::new(button.builder()));

    util::set_root_and_loop(window, ui, root_widget);
}
