pub mod mouse;
pub mod keyboard;

use glutin;

use event::{Target, UiEventHandler};
use input::mouse::{MouseMoved, MouseButton, MouseWheel};
use input::keyboard::{KeyboardInput, ReceivedCharacter};
use util::Point;
use ui::Ui;

#[derive(Clone)]
pub struct InputEvent(pub glutin::Event);

pub fn handle_input(event: &InputEvent, ui: &mut Ui) {
    let InputEvent(event) = event.clone();
    match event {
        glutin::Event::Closed => {
            ui.close();
        }
        glutin::Event::MouseWheel(mouse_scroll_delta, _) => {
            event!(Target::Ui, MouseWheel(mouse_scroll_delta));
        }
        glutin::Event::MouseInput(state, button) => {
            event!(Target::Ui, MouseButton(state, button));
        }
        glutin::Event::MouseMoved(x, y) => {
            let point = Point::new(x as f64, y as f64);
            event!(Target::Ui, MouseMoved(point));
        }
        glutin::Event::KeyboardInput(state, scan_code, maybe_keycode) => {
            let key_input = KeyboardInput(state, scan_code, maybe_keycode);
            event!(Target::Ui, key_input);
        }
        glutin::Event::ReceivedCharacter(char) => {
            event!(Target::Ui, ReceivedCharacter(char));
        }
        _ => (),
    }
}

pub struct EscKeyCloseHandler;
impl UiEventHandler<KeyboardInput> for EscKeyCloseHandler {
    fn handle(&mut self, event: &KeyboardInput, ui: &mut Ui) {
        if let &KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape)) = event {
            ui.close();
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
impl UiEventHandler<KeyboardInput> for DebugSettingsHandler {
    fn handle(&mut self, event: &KeyboardInput, ui: &mut Ui) {
        if let &KeyboardInput(ElementState::Released, _, Some(glutin::VirtualKeyCode::F1)) = event {
            self.debug_on = !self.debug_on;
            ui.set_debug_draw_bounds(self.debug_on);
        }
        if let &KeyboardInput(ElementState::Released, _, Some(glutin::VirtualKeyCode::F2)) = event {
            ui.debug_constraints();
        }
        if let &KeyboardInput(ElementState::Released, _, Some(glutin::VirtualKeyCode::F3)) = event {
            ui.debug_widget_positions();
        }
        if let &KeyboardInput(ElementState::Released, _, Some(glutin::VirtualKeyCode::F4)) = event {
            ui.debug_variables();
        }
    }
}
