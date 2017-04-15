use text_layout::Align;

use widget::{WidgetBuilder, WidgetBuilderCore, BuildWidget};
use widget::property::states::*;
use ui::{WidgetAttachedEvent, WidgetDetachedEvent};
use input::keyboard::{WidgetReceivedCharacter, KeyboardInputEvent};
use drawable::rect::{RectDrawable, RectStyleable};
use drawable::text::{TextDrawable, TextStyleable};
use event::{Target, WidgetEventArgs};
use color::*;

const BACKSPACE: char = '\u{8}';

fn edit_text_handle_char(event: &WidgetReceivedCharacter, args: WidgetEventArgs) {
    let &WidgetReceivedCharacter(char) = event;
    let mut text = args.widget.drawable::<TextDrawable>().unwrap().text.clone();
    match char {
        BACKSPACE => {
            text.pop();
        }
        _ => {
            text.push(char);
            let drawable = args.widget.drawable::<TextDrawable>().unwrap();
            if !drawable.text_fits(&text, args.widget.layout.bounds()) {
                text.pop();
            }
        }
    }
    args.widget.update(|state: &mut TextDrawable| {
        state.text = text.clone()
    });
    args.queue.push(Target::Widget(args.widget.id), TextUpdated(text.clone()));
}

pub struct TextUpdated(pub String);

pub fn text_change_handle(event: &TextUpdated, args: WidgetEventArgs) {
    args.widget.update(|state: &mut TextDrawable| state.text = event.0.clone());
}

pub struct EditTextBuilder {
    pub widget: WidgetBuilder,
    pub text_widget: WidgetBuilder,
}
impl AsMut<WidgetBuilder> for EditTextBuilder {
    fn as_mut(&mut self) -> &mut WidgetBuilder {
        &mut self.widget
    }
}
impl BuildWidget for EditTextBuilder {
    fn build(mut self) -> WidgetBuilder {
        self.widget.add_child(self.text_widget);
        self.widget
    }
}

impl EditTextBuilder {
    pub fn new() -> Self {

        let default_border = Some((1.0, GRAY));
        let focused_border = Some((1.0, BLUE));
        let rect_style = style!(
            RectStyleable::Border: selector!(default_border, FOCUSED: focused_border),
            RectStyleable::CornerRadius: Some(3.0));
        let mut widget = WidgetBuilder::new();
        widget
            .set_drawable_with_style(RectDrawable::new(), rect_style)
            .add_handler_fn(|_: &WidgetAttachedEvent, args| {
                args.queue.push(Target::Ui, KeyboardInputEvent::AddFocusable(args.widget.id));
            })
            .add_handler_fn(|_: &WidgetDetachedEvent, args| {
                args.queue.push(Target::Ui, KeyboardInputEvent::RemoveFocusable(args.widget.id));
            })
            .make_focusable();


        let text_style = style!(TextStyleable::VertAlign: Align::Start);
        let mut text_widget = WidgetBuilder::new();
        text_widget
            .set_drawable_with_style(TextDrawable::default(), text_style)
            .add_handler_fn(edit_text_handle_char)
            .add_handler_fn(text_change_handle);
        text_widget.layout().bound_left(&widget).padding(5.0);
        text_widget.layout().bound_right(&widget).padding(5.0);

        EditTextBuilder {
            widget: widget,
            text_widget: text_widget,
        }
    }

    pub fn on_text_changed<F>(&mut self, callback: F) -> &mut Self
        where F: Fn(&TextUpdated, WidgetEventArgs) + 'static
    {
        self.text_widget.add_handler_fn(callback);
        self
    }
}