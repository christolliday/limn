extern crate limn;
#[macro_use]
extern crate limn_layout;
extern crate text_layout;

mod util;

use text_layout::{Align, Wrap};

use limn::prelude::*;

use limn::widgets::button::{ToggleButtonBuilder, ToggleEvent};
use limn::widgets::edit_text::EditTextBuilder;
use limn::drawable::text::TextDrawable;

enum EditTextSettingsEvent {
    Align(Align),
    Wrap(Wrap),
}
struct EditTextSettingsHandler;
impl WidgetEventHandler<EditTextSettingsEvent> for EditTextSettingsHandler {
    fn handle(&mut self, event: &EditTextSettingsEvent, mut args: WidgetEventArgs) {
        args.widget.update(|drawable: &mut TextDrawable| {
            match *event {
                EditTextSettingsEvent::Align(align) => drawable.align = align,
                EditTextSettingsEvent::Wrap(wrap) => drawable.wrap = wrap,
            }
        });
    }
}

fn main() {
    let app = util::init_default("Limn edit text demo");
    let mut root = WidgetBuilder::new("root");

    let mut content_widget = WidgetBuilder::new("content");
    root.layout().add(min_size(Size::new(500.0, 500.0)));
    content_widget.layout().add(match_layout(&root).padding(20.0));

    let mut edit_text_box = EditTextBuilder::new();
    edit_text_box.text_widget.add_handler(EditTextSettingsHandler);

    let edit_text_ref = edit_text_box.text_widget.widget_ref();
    let mut h_align_button = ToggleButtonBuilder::new();
    h_align_button
        .set_text("Right Align", "Left Align")
        .on_toggle(move |event, _| {
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
    let mut v_align_button = ToggleButtonBuilder::new();
    v_align_button
        .set_text("Wrap Word", "Wrap Char")
        .on_toggle(move |event, _| {
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
