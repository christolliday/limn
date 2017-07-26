#[macro_use]
extern crate limn;
#[macro_use]
extern crate limn_layout;
extern crate text_layout;

mod util;

use text_layout::Align;

use limn::prelude::*;

use limn::widgets::button::{ToggleButtonBuilder, ToggleEvent};
use limn::widgets::edit_text::EditTextBuilder;
use limn::drawable::text::TextDrawable;

enum EditTextSettingsEvent {
    LeftAlign,
    RightAlign,
    TopAlign,
    BottomAlign,
}
struct EditTextSettingsHandler;
impl WidgetEventHandler<EditTextSettingsEvent> for EditTextSettingsHandler {
    fn handle(&mut self, event: &EditTextSettingsEvent, mut args: WidgetEventArgs) {
        args.widget.update(|drawable: &mut TextDrawable| {
            match *event {
                EditTextSettingsEvent::LeftAlign => drawable.align = Align::Start,
                EditTextSettingsEvent::RightAlign => drawable.align = Align::End,
                EditTextSettingsEvent::TopAlign => drawable.vertical_align = Align::Start,
                EditTextSettingsEvent::BottomAlign => drawable.vertical_align = Align::End,
            }
        });
    }
}

fn main() {
    let app = util::init_default("Limn edit text demo");
    util::load_default_font();

    let mut root_widget = WidgetBuilder::new();
    layout!(root_widget: min_size(Size::new(300.0, 300.0)));

    let mut edit_text_box = EditTextBuilder::new();
    edit_text_box.text_widget.add_handler(EditTextSettingsHandler);
    let edit_text_id = edit_text_box.text_widget.id();

    let mut h_align_button = ToggleButtonBuilder::new();
    h_align_button
        .set_text("Right Align", "Left Align")
        .on_toggle(move |event, _| {
            match *event {
                ToggleEvent::On => {
                    event!(Target::Widget(edit_text_id), EditTextSettingsEvent::RightAlign);
                },
                ToggleEvent::Off => {
                    event!(Target::Widget(edit_text_id), EditTextSettingsEvent::LeftAlign);
                },
            }
        });

    let mut v_align_button = ToggleButtonBuilder::new();
    v_align_button
        .set_text("Bottom Align", "Top Align")
        .on_toggle(move |event, _| {
            match *event {
                ToggleEvent::On => {
                    event!(Target::Widget(edit_text_id), EditTextSettingsEvent::BottomAlign);
                },
                ToggleEvent::Off => {
                    event!(Target::Widget(edit_text_id), EditTextSettingsEvent::TopAlign);
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
