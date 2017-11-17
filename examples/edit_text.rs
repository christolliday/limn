#[macro_use]
extern crate limn;

mod util;

use limn::prelude::*;

use limn::widgets::button::{ToggleButtonStyle, ToggleEvent};
use limn::widgets::edit_text::EditTextBuilder;
use limn::draw::text::TextState;

enum EditTextSettingsEvent {
    Align(Align),
    Wrap(Wrap),
}
struct EditTextSettingsHandler;
impl EventHandler<EditTextSettingsEvent> for EditTextSettingsHandler {
    fn handle(&mut self, event: &EditTextSettingsEvent, mut args: EventArgs) {
        args.widget.update(|draw_state: &mut TextState| {
            match *event {
                EditTextSettingsEvent::Align(align) => draw_state.align = align,
                EditTextSettingsEvent::Wrap(wrap) => draw_state.wrap = wrap,
            }
        });
    }
}

fn main() {
    let window_builder = glutin::WindowBuilder::new()
        .with_title("Limn edit text demo")
        .with_min_dimensions(100, 100);
    let app = util::init(window_builder);
    let mut root = WidgetBuilder::new("root");

    let mut content_widget = WidgetBuilder::new("content");
    root.layout().add(min_size(Size::new(500.0, 500.0)));
    content_widget.layout().add(match_layout(&root).padding(20.0));

    let mut edit_text_box = EditTextBuilder::new();
    edit_text_box.text_widget.add_handler(EditTextSettingsHandler);

    let edit_text_ref = edit_text_box.text_widget.widget_ref();
    let mut h_align_button = ToggleButtonStyle::default();
    h_align_button.toggle_text("Right Align", "Left Align");
    let mut h_align_button = WidgetBuilder::from_component_style(h_align_button);
    h_align_button.add_handler(move |event: &ToggleEvent, _: EventArgs| {
        match *event {
            ToggleEvent::On => {
                edit_text_ref.event(EditTextSettingsEvent::Align(Align::End));
            },
            ToggleEvent::Off => {
                edit_text_ref.event(EditTextSettingsEvent::Align(Align::Start));
            },
        }
    });

    let edit_text_ref = edit_text_box.text_widget.widget_ref();

    let mut v_align_button = ToggleButtonStyle::default();
    v_align_button.toggle_text("Wrap Word", "Wrap Char");
    let mut v_align_button = WidgetBuilder::from_component_style(v_align_button);
    v_align_button.add_handler(move |event: &ToggleEvent, _: EventArgs| {
        match *event {
            ToggleEvent::On => {
                edit_text_ref.event(EditTextSettingsEvent::Wrap(Wrap::Whitespace));
            },
            ToggleEvent::Off => {
                edit_text_ref.event(EditTextSettingsEvent::Wrap(Wrap::Character));
            },
        }
    });

    h_align_button.layout().add(constraints![
        align_top(&content_widget),
        align_left(&content_widget),
    ]);

    v_align_button.layout().add(constraints![
        align_top(&content_widget),
        align_right(&content_widget),
    ]);

    edit_text_box.layout().add(constraints![
        below(&h_align_button).padding(20.0),
        below(&v_align_button).padding(20.0),
        align_bottom(&content_widget),
        align_left(&content_widget),
        align_right(&content_widget),
    ]);

    content_widget
        .add_child(h_align_button)
        .add_child(v_align_button)
        .add_child(edit_text_box);

    root.add_child(content_widget);
    app.main_loop(root);
}
