extern crate limn;

mod util;

use limn::widget::builder::WidgetBuilder;
use limn::widgets::button::ToggleButtonBuilder;
use limn::widgets::text::{self, TextStyleField};
use limn::widgets::edit_text::EditTextBuilder;
use limn::widget::style::Value;
use limn::util::Dimensions;
use limn::color::*;

fn main() {
    let (window, ui) = util::init_default("Limn button demo");
    util::load_default_font();

    let mut root_widget = WidgetBuilder::new();
    root_widget.layout.min_dimensions(Dimensions {
        width: 300.0,
        height: 300.0,
    });

    let text_style = vec![TextStyleField::Text(Value::Single("I believe in \
                                                              reincarnation.\nThat's why I eat \
                                                              Jello.\nIt's good for the stomach"
                              .to_owned())),
                          TextStyleField::BackgroundColor(Value::Single(WHITE))];
    let text_drawable = text::text_drawable(text_style.clone());

    let text_widget = WidgetBuilder::new()
        .set_drawable(text_drawable)
        .set_debug_name("text");

    let mut button = ToggleButtonBuilder::new()
        .set_text("ON", "OFF")
        .widget;
    button.layout.center_horizontal(&root_widget);
    button.layout.below(&text_widget, Some(20.0));

    let mut edit_text = EditTextBuilder::new().widget;
    edit_text.layout.below(&button, Some(20.0));
    edit_text.layout.align_bottom(&root_widget, Some(20.0));
    edit_text.layout.height(100.0);

    root_widget.add_child(text_widget);
    root_widget.add_child(button);
    root_widget.add_child(edit_text);

    util::set_root_and_loop(window, ui, root_widget);
}
