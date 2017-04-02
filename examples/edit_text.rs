extern crate limn;
extern crate text_layout;

mod util;

use text_layout::Align;

use limn::event::{Target, WidgetEventHandler, WidgetEventArgs};
use limn::widget::{WidgetBuilder, WidgetBuilderCore};
use limn::widgets::button::{ToggleButtonBuilder, ToggleEvent};
use limn::widgets::edit_text::EditTextBuilder;
use limn::drawable::text::TextDrawable;
use limn::util::Dimensions;

enum EditTextSettingsEvent {
    LeftAlign,
    RightAlign,
    TopAlign,
    BottomAlign,
}
struct EditTextSettingsHandler;
impl WidgetEventHandler<EditTextSettingsEvent> for EditTextSettingsHandler {
    fn handle(&mut self, event: &EditTextSettingsEvent, args: WidgetEventArgs) {
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
    let (window, ui) = util::init_default("Limn edit text demo");
    util::load_default_font();

    let mut root_widget = WidgetBuilder::new();
    root_widget.layout().min_dimensions(Dimensions {
        width: 300.0,
        height: 300.0,
    });

    let mut edit_text_box = EditTextBuilder::new();
    edit_text_box.text_widget.add_handler(EditTextSettingsHandler);
    let edit_text_id = edit_text_box.text_widget.id();

    let mut h_align_button = ToggleButtonBuilder::new();
    h_align_button
        .set_text("Right Align", "Left Align")
        .on_toggle(move |event, args| {
            match *event {
                ToggleEvent::On => {
                    args.queue.push(Target::Widget(edit_text_id), EditTextSettingsEvent::RightAlign);
                },
                ToggleEvent::Off => {
                    args.queue.push(Target::Widget(edit_text_id), EditTextSettingsEvent::LeftAlign);
                },
            }
        });

    let mut v_align_button = ToggleButtonBuilder::new();
    v_align_button
        .set_text("Bottom Align", "Top Align")
        .on_toggle(move |event, args| {
            match *event {
                ToggleEvent::On => {
                    args.queue.push(Target::Widget(edit_text_id), EditTextSettingsEvent::BottomAlign);
                },
                ToggleEvent::Off => {
                    args.queue.push(Target::Widget(edit_text_id), EditTextSettingsEvent::TopAlign);
                },
            }
        });

    h_align_button.layout().align_top(&root_widget.layout()).padding(20.0);
    h_align_button.layout().align_left(&root_widget.layout()).padding(20.0);
    v_align_button.layout().align_top(&root_widget.layout()).padding(20.0);
    v_align_button.layout().align_right(&root_widget.layout()).padding(20.0);

    edit_text_box.layout().below(&h_align_button.layout()).padding(20.0);
    edit_text_box.layout().align_bottom(&root_widget.layout()).padding(20.0);
    edit_text_box.layout().align_left(&root_widget.layout()).padding(20.0);
    edit_text_box.layout().align_right(&root_widget.layout()).padding(20.0);

    root_widget.add_child(h_align_button);
    root_widget.add_child(v_align_button);
    root_widget.add_child(edit_text_box);

    util::set_root_and_loop(window, ui, root_widget);
}
