use widget::{EventArgs, EventHandler};
use resources::WidgetId;
use ui::mouse::ClickEvent;
use ui::queue::EventAddress;
use ui;
use ui::event::*;


pub struct FocusHandler {
    focused: Option<WidgetId>,
}
impl FocusHandler {
    pub fn new() -> Self {
        FocusHandler {
            focused: None,
        }
    }
}
impl ui::EventHandler<KeyboardInputEvent> for FocusHandler {
    fn handle(&mut self, event: &KeyboardInputEvent, mut args: ui::EventArgs) {
        match event {
            &KeyboardInputEvent::FocusChange(new_focus) => {
                self.focused = new_focus;
            }
            &KeyboardInputEvent::KeyboardInput(ref key_input) => {
                if let Some(focused) = self.focused {
                    let &KeyboardInput(state, scan_code, maybe_keycode) = key_input;
                    let event = WidgetKeyboardInput(state, scan_code, maybe_keycode);
                    args.event_queue.push(EventAddress::Widget(focused), event);
                }
            }
        }
    }
}

enum KeyboardInputEvent {
    FocusChange(Option<WidgetId>),
    KeyboardInput(KeyboardInput),
}

pub struct KeyboardForwarder;
impl ui::EventHandler<KeyboardInput> for KeyboardForwarder {
    fn handle(&mut self, event: &KeyboardInput, mut args: ui::EventArgs) {
        args.event_queue.push(EventAddress::Ui, KeyboardInputEvent::KeyboardInput(event.clone()));
    }
}

pub struct WidgetFocusHandler;
impl EventHandler<ClickEvent> for WidgetFocusHandler {
    fn handle(&mut self, _: &ClickEvent, mut args: EventArgs) {
        args.event_queue.push(EventAddress::Ui, KeyboardInputEvent::FocusChange(Some(args.widget_id)));
    }
}