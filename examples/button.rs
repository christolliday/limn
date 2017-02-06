extern crate limn;

mod util;

use limn::widget::builder::WidgetBuilder;
use limn::widgets::button::ToggleButtonBuilder;

fn main() {
    let (window, ui, event_queue) = util::init_default("Limn button demo");
    let font_id = util::load_default_font();

    let mut root_widget = WidgetBuilder::new();
    let mut button = ToggleButtonBuilder::new()
        .set_text("ON", "OFF")
        .widget;
    button.layout.center(&root_widget);
    button.layout.bound_by(&root_widget, Some(50.0));
    root_widget.add_child(Box::new(button));

    util::set_root_and_loop(window, ui, root_widget, event_queue, vec!{});
}
