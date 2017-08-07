use text_layout::Align;

use widget::{Widget, BuildWidget};
use widget::property::states::*;
use ui::{WidgetAttachedEvent, WidgetDetachedEvent};
use input::keyboard::{WidgetReceivedCharacter, KeyboardInputEvent};
use drawable::rect::{RectDrawable, RectStyleable};
use drawable::text::{TextDrawable, TextStyleable};
use event::{Target, WidgetEventArgs};
use color::*;

const BACKSPACE: char = '\u{8}';

fn edit_text_handle_char(event: &WidgetReceivedCharacter, mut args: WidgetEventArgs) {
    let &WidgetReceivedCharacter(char) = event;
    let text = {
        let bounds = args.widget.bounds();
        let drawable = args.widget.drawable();
        let text_drawable = drawable.downcast_ref::<TextDrawable>().unwrap();
        let mut text = text_drawable.text.clone();
        match char {
            BACKSPACE => {
                text.pop();
            }
            _ => {
                text.push(char);
                if !text_drawable.text_fits(&text, bounds) {
                    text.pop();
                }
            }
        }
        text
    };
    args.widget.update(|state: &mut TextDrawable| {
        state.text = text.clone()
    });
    args.widget.event(TextUpdated(text.clone()));
}

pub struct TextUpdated(pub String);

pub fn text_change_handle(event: &TextUpdated, mut args: WidgetEventArgs) {
    args.widget.update(|state: &mut TextDrawable| state.text = event.0.clone());
}

pub struct EditTextBuilder {
    pub widget: Widget,
    pub text_widget: Widget,
}

impl EditTextBuilder {
    pub fn new() -> Self {
        let default_border = Some((1.0, GRAY));
        let focused_border = Some((1.0, BLUE));
        let rect_style = style!(
            RectStyleable::Border: selector!(default_border, FOCUSED: focused_border),
            RectStyleable::CornerRadius: Some(3.0));
        let mut widget = Widget::new();
        widget
            .set_drawable_with_style(RectDrawable::new(), rect_style)
            .add_handler_fn(|_: &WidgetAttachedEvent, args| {
                event!(Target::Ui, KeyboardInputEvent::AddFocusable(args.widget));
            })
            .add_handler_fn(|_: &WidgetDetachedEvent, args| {
                event!(Target::Ui, KeyboardInputEvent::RemoveFocusable(args.widget));
            })
            .make_focusable();


        let text_style = style!(TextStyleable::VertAlign: Align::Start);
        let mut text_widget = Widget::new();
        text_widget
            .set_drawable_with_style(TextDrawable::default(), text_style)
            .add_handler_fn(edit_text_handle_char)
            .add_handler_fn(text_change_handle);
        layout!(text_widget:
            bound_left(&widget).padding(5.0),
            bound_right(&widget).padding(5.0));

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

widget_builder!(EditTextBuilder);
impl BuildWidget for EditTextBuilder {
    fn build(mut self) -> Widget {
        self.widget.add_child(self.text_widget);
        self.widget
    }
}
