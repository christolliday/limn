use text_layout::Align;

use widget::{WidgetBuilder, EventHandler, EventArgs, CallbackHandler};
use widget::WidgetBuilderCore;
use widget::property::PropChangeHandler;
use widget::property::states::*;
use widget::style::{Value, Selector};
use ui::{WidgetAttachedEvent, WidgetDetachedEvent};
use input::keyboard::{WidgetFocusHandler, WidgetReceivedCharacter, KeyboardInputEvent};
use drawable::rect::{RectDrawable, RectStyleField};
use drawable::text::{TextDrawable, TextStyleField};
use event::Target;
use color::*;

const BACKSPACE: char = '\u{8}';

pub struct EditTextKeyboardHandler;
impl EventHandler<WidgetReceivedCharacter> for EditTextKeyboardHandler {
    fn handle(&mut self, event: &WidgetReceivedCharacter, args: EventArgs) {
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
}
pub struct TextUpdated(pub String);

pub struct TextChangeHandler;
impl EventHandler<TextUpdated> for TextChangeHandler {
    fn handle(&mut self, event: &TextUpdated, args: EventArgs) {
        args.widget.update(|state: &mut TextDrawable| state.text = event.0.clone());
    }
}

pub struct EditTextBuilder {
    pub widget: WidgetBuilder,
}
impl AsMut<WidgetBuilder> for EditTextBuilder {
    fn as_mut(&mut self) -> &mut WidgetBuilder {
        &mut self.widget
    }
}

impl EditTextBuilder {
    pub fn new() -> Self {

        let default_border = Some((1.0, GRAY));
        let focused_border = Some((1.0, BLUE));
        let rect_style = {
            let mut selector = Selector::new(default_border);
            selector.insert(&FOCUSED, focused_border);
            vec![
                RectStyleField::Border(Value::Selector(selector)),
                RectStyleField::CornerRadius(Value::Single(Some(3.0)))
            ]
        };
        let mut widget = WidgetBuilder::new();
        widget
            .set_drawable_with_style(RectDrawable::new(), rect_style)
            .add_handler(CallbackHandler::new(|_: &WidgetAttachedEvent, args| {
                args.queue.push(Target::Ui, KeyboardInputEvent::AddFocusable(args.widget.id));
            }))
            .add_handler(CallbackHandler::new(|_: &WidgetDetachedEvent, args| {
                args.queue.push(Target::Ui, KeyboardInputEvent::RemoveFocusable(args.widget.id));
            }))
            .add_handler(WidgetFocusHandler)
            .add_handler(PropChangeHandler);


        let text_style = vec![TextStyleField::VertAlign(Value::Single(Align::Start))];
        let mut text_widget = WidgetBuilder::new();
        text_widget
            .set_drawable_with_style(TextDrawable::default(), text_style)
            .add_handler(EditTextKeyboardHandler)
            .add_handler(TextChangeHandler)
            .add_handler(PropChangeHandler);
        text_widget.layout().bound_left(&widget.layout()).padding(5.0);
        text_widget.layout().bound_right(&widget.layout()).padding(5.0);

        widget.add_child(text_widget);
        EditTextBuilder {
            widget: widget,
        }
    }

    pub fn on_text_changed<F>(&mut self, callback: F) -> &mut Self
        where F: Fn(&TextUpdated, &mut EventArgs) + 'static
    {
        {
            let edit_text = self.widget.children.get_mut(0).unwrap();
            edit_text.add_handler(CallbackHandler::new(callback));
        }
        self
    }
}