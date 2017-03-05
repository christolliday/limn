use widget::{EventHandler, EventArgs};
use input::keyboard::{WidgetFocusHandler, WidgetReceivedCharacter};
use widget::property::PropsChangeEventHandler;
use widget::builder::WidgetBuilder;
use widgets::{primitives, text};
use widgets::text::TextDrawState;

pub struct EditTextKeyboardHandler {
    text: String,
}
impl EditTextKeyboardHandler {
    pub fn new() -> Self {
        EditTextKeyboardHandler {
            text: "".to_owned(),
        }
    }
}
impl EventHandler<WidgetReceivedCharacter> for EditTextKeyboardHandler {
    fn handle(&mut self, event: &WidgetReceivedCharacter, args: EventArgs) {
        //println!("widget key input: {:?}", event);
        let &WidgetReceivedCharacter(char) = event;
        match char {
            '\u{8}' => {
                self.text.pop();
            }
            _ => {
                self.text.push(char);
            }
        }
        if let Some(drawable) = args.widget.drawable.as_mut() {
            drawable.update(|state: &mut TextDrawState| state.text = self.text.clone());
        }
    }
}


pub struct EditTextBuilder {
    pub widget: WidgetBuilder,
}
impl EditTextBuilder {
    pub fn new() -> Self {

        let mut widget = WidgetBuilder::new()
            .set_drawable(primitives::rect_drawable(vec![]))
            .add_handler(WidgetFocusHandler)
            .add_handler(PropsChangeEventHandler);

        let text_widget = WidgetBuilder::new()
            .set_drawable(text::text_drawable(vec![]))
            .add_handler(EditTextKeyboardHandler::new())
            .add_handler(PropsChangeEventHandler);
        
        widget.add_child(text_widget);
        EditTextBuilder {
            widget: widget,
        }
    }
}