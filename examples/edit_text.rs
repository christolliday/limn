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

    let mut root_widget = Widget::new();
    layout!(root_widget: min_size(Size::new(300.0, 300.0)));

    let mut edit_text_box = EditTextBuilder::new();
    edit_text_box.text_widget.add_handler(EditTextSettingsHandler);

    let edit_text_ref = edit_text_box.text_widget.clone();
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

    let edit_text_ref = edit_text_box.text_widget.clone();
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

    layout!(h_align_button:
        align_top(&root_widget).padding(20.0),
        align_left(&root_widget).padding(20.0));

    layout!(v_align_button:
        align_top(&root_widget).padding(20.0),
        align_right(&root_widget).padding(20.0));

    layout!(edit_text_box:
        below(&h_align_button).padding(20.0),
        align_bottom(&root_widget).padding(20.0),
        align_left(&root_widget).padding(20.0),
        align_right(&root_widget).padding(20.0));

    root_widget
        .add_child(h_align_button)
        .add_child(v_align_button)
        .add_child(edit_text_box);

    util::set_root_and_loop(app, root_widget);
}
