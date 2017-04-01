pub mod mouse;
pub mod keyboard;

use glutin;

use ui;
use event::Target;
use input::mouse::{MouseMoved, MouseButton, MouseWheel};
use input::keyboard::{KeyboardInput, ReceivedCharacter};
use util::Point;

#[derive(Clone)]
pub struct InputEvent(pub glutin::Event);

pub fn handle_input(event: &InputEvent, args: ui::EventArgs) {
    let queue = args.queue;
    let InputEvent(event) = event.clone();
    match event {
        glutin::Event::Closed => {
            args.ui.close();
        }
        glutin::Event::MouseWheel(mouse_scroll_delta, _) => {
            queue.push(Target::Ui, MouseWheel(mouse_scroll_delta));
        }
        glutin::Event::MouseInput(state, button) => {
            queue.push(Target::Ui, MouseButton(state, button));
        }
        glutin::Event::MouseMoved(x, y) => {
            let point = Point::new(x as f64, y as f64);
            queue.push(Target::Ui, MouseMoved(point));
        }
        glutin::Event::KeyboardInput(state, scan_code, maybe_keycode) => {
            let key_input = KeyboardInput(state, scan_code, maybe_keycode);
            queue.push(Target::Ui, key_input);
        }
        glutin::Event::ReceivedCharacter(char) => {
            queue.push(Target::Ui, ReceivedCharacter(char));
        }
        _ => (),
    }
}

pub struct EscKeyCloseHandler;
impl ui::EventHandler<KeyboardInput> for EscKeyCloseHandler {
    fn handle(&mut self, event: &KeyboardInput, args: ui::EventArgs) {
        if let &KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape)) = event {
            args.ui.close();
        }
    }
}
pub struct DebugSettingsHandler {
    debug_on: bool
}
impl DebugSettingsHandler {
    pub fn new() -> Self {
        DebugSettingsHandler {
            debug_on: false,
        }
    }
}
use glutin::ElementState;
impl ui::EventHandler<KeyboardInput> for DebugSettingsHandler {
    fn handle(&mut self, event: &KeyboardInput, args: ui::EventArgs) {
        if let &KeyboardInput(ElementState::Released, _, Some(glutin::VirtualKeyCode::F1)) = event {
            self.debug_on = !self.debug_on;
            args.ui.set_debug_draw_bounds(self.debug_on);
        }
    }
}
