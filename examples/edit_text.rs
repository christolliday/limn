extern crate limn;

mod util;

use limn::widget::WidgetBuilder;
use limn::widgets::button::ToggleButtonBuilder;
use limn::widgets::edit_text::EditTextBuilder;
use limn::util::Dimensions;

fn main() {
    let (window, ui) = util::init_default("Limn edit text demo");
    util::load_default_font();

    let mut root_widget = WidgetBuilder::new();
    root_widget.layout.min_dimensions(Dimensions {
        width: 300.0,
        height: 300.0,
    });

    let mut button = ToggleButtonBuilder::new()
        .set_text("Left", "Right")
        .widget;
    button.layout.align_top(&root_widget, Some(20.0));
    button.layout.center_horizontal(&root_widget);

    let mut edit_text = EditTextBuilder::new().widget;
    edit_text.layout.below(&button, Some(20.0));
    edit_text.layout.align_bottom(&root_widget, Some(20.0));
    edit_text.layout.align_left(&root_widget, Some(20.0));
    edit_text.layout.align_right(&root_widget, Some(20.0));

    root_widget.add_child(button);
    root_widget.add_child(edit_text);

    util::set_root_and_loop(window, ui, root_widget);
}
