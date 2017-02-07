extern crate limn;

mod util;

use limn::widget::builder::WidgetBuilder;
use limn::widgets::button::ToggleButtonBuilder;
use limn::widgets::text::{self, TextStyle, TextStyleField};
use limn::widget::style::Value;
use limn::color::*;

fn main() {
    let (window, ui, event_queue) = util::init_default("Limn button demo");
    let font_id = util::load_default_font();

    let mut root_widget = WidgetBuilder::new();

    let text_fields = vec!{
        TextStyleField::text(Value::Single("hello, can anybody hear me".to_owned())),
        TextStyleField::background_color(Value::Single(WHITE)),
    };
    let text_style = TextStyle::from(text_fields);
    let text_drawable = text::text_drawable(text_style);
    let text_dims = text::measure_dims_no_wrap(&text_drawable);

    let mut text_widget = WidgetBuilder::new()
        .set_drawable(text_drawable)
        .set_debug_name("text");
    text_widget.layout.dimensions(text_dims);
    //text_widget.layout.center(&root_widget);

    let mut button = ToggleButtonBuilder::new()
        .set_text("ON", "OFF")
        .widget;
    button.layout.center_horizontal(&root_widget);
    button.layout.align_bottom(&root_widget, Some(20.0));
    button.layout.below(&text_widget, Some(20.0));

    root_widget.add_child(text_widget);
    root_widget.add_child(button);

    util::set_root_and_loop(window, ui, root_widget, event_queue, vec!{});
}
