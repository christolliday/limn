use widget::{EventArgs, EventHandler};
use resources::WidgetId;
use input::mouse::ClickEvent;
use ui::queue::Target;
use ui;

use glutin;

#[derive(Clone, Debug)]
pub struct ReceivedCharacter(pub char);
#[derive(Clone, Debug)]
pub struct KeyboardInput(pub glutin::ElementState, pub glutin::ScanCode, pub Option<glutin::VirtualKeyCode>);
#[derive(Debug)]
pub struct WidgetKeyboardInput(pub glutin::ElementState, pub glutin::ScanCode, pub Option<glutin::VirtualKeyCode>);
#[derive(Debug)]
pub struct WidgetReceivedCharacter(pub char);

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
                    args.event_queue.push(Target::SubTree(focused), event);
                }
            }
            &KeyboardInputEvent::ReceivedCharacter(ref received_char) => {
                if let Some(focused) = self.focused {
                    let &ReceivedCharacter(char) = received_char;
                    let event = WidgetReceivedCharacter(char);
                    args.event_queue.push(Target::SubTree(focused), event);
                }
            }
        }
    }
}

enum KeyboardInputEvent {
    FocusChange(Option<WidgetId>),
    KeyboardInput(KeyboardInput),
    ReceivedCharacter(ReceivedCharacter),
}

pub struct KeyboardForwarder;
impl ui::EventHandler<KeyboardInput> for KeyboardForwarder {
    fn handle(&mut self, event: &KeyboardInput, mut args: ui::EventArgs) {
        args.event_queue.push(Target::Ui, KeyboardInputEvent::KeyboardInput(event.clone()));
    }
}
pub struct KeyboardCharForwarder;
impl ui::EventHandler<ReceivedCharacter> for KeyboardCharForwarder {
    fn handle(&mut self, event: &ReceivedCharacter, mut args: ui::EventArgs) {
        args.event_queue.push(Target::Ui, KeyboardInputEvent::ReceivedCharacter(event.clone()));
    }
}

pub struct WidgetFocusHandler;
impl EventHandler<ClickEvent> for WidgetFocusHandler {
    fn handle(&mut self, _: &ClickEvent, mut args: EventArgs) {
        args.event_queue.push(Target::Ui, KeyboardInputEvent::FocusChange(Some(args.widget.id)));
    }
}