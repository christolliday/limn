use widget::{WidgetBuilder, EventHandler, EventArgs, CallbackHandler};
use widget::WidgetBuilderCore;
use widget::property::PropChangeHandler;
use ui::{WidgetAttachedEvent, WidgetDetachedEvent};
use input::keyboard::{WidgetFocusHandler, WidgetReceivedCharacter, KeyboardInputEvent};
use drawable::rect::RectDrawable;
use drawable::text::TextDrawable;
use event::Target;

pub struct EditTextKeyboardHandler;
impl EventHandler<WidgetReceivedCharacter> for EditTextKeyboardHandler {
    fn handle(&mut self, event: &WidgetReceivedCharacter, args: EventArgs) {
        let &WidgetReceivedCharacter(char) = event;
        let mut text = args.widget.drawable::<TextDrawable>().unwrap().text.clone();
        match char {
            '\u{8}' => {
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

use drawable::text::TextStyleField;
use widget::style::Value;
use text_layout::Align;
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

        let mut widget = WidgetBuilder::new();
        widget
            .set_drawable(RectDrawable::new())
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