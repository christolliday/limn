use widget::{EventHandler, EventArgs};
use ui::keyboard::WidgetFocusHandler;
use widget::property::PropsChangeEventHandler;
use widget::builder::WidgetBuilder;
use widgets::{primitives, text};
use ui::event::*;

pub struct EditTextKeyboardHandler;
impl EventHandler<WidgetKeyboardInput> for EditTextKeyboardHandler {
    fn handle(&mut self, event: &WidgetKeyboardInput, _: EventArgs) {
        println!("widget key input: {:?}", event);
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
            .add_handler(EditTextKeyboardHandler)
            .add_handler(PropsChangeEventHandler);

        let text_widget = WidgetBuilder::new()
            .set_drawable(text::text_drawable(vec![]))
            .add_handler(PropsChangeEventHandler);
        
        widget.add_child(text_widget);
        EditTextBuilder {
            widget: widget,
        }
    }
}