pub mod mouse;
pub mod keyboard;

use glutin;

use event::{Target, UiEventHandler};
use input::mouse::{MouseMoved, MouseButton, MouseWheel};
use input::keyboard::{KeyboardInput, ReceivedCharacter};
use util::Point;
use ui::Ui;
use app::App;

#[derive(Clone)]
pub struct InputEvent(pub glutin::WindowEvent);

impl App {
    pub fn add_input_handlers(&mut self) {
        self.add_handler_fn(|event: &InputEvent, ui| {
            let InputEvent(event) = event.clone();
            match event {
                glutin::WindowEvent::Closed => {
                    ui.close();
                }
                glutin::WindowEvent::MouseWheel { delta, .. } => {
                    event!(Target::Ui, MouseWheel(delta));
                }
                glutin::WindowEvent::MouseInput { state, button, .. } => {
                    event!(Target::Ui, MouseButton(state, button));
                }
                glutin::WindowEvent::MouseMoved { position, .. } => {
                    let point = Point::new(position.0 as f32, position.1 as f32);
                    event!(Target::Ui, MouseMoved(point));
                }
                glutin::WindowEvent::KeyboardInput { input, .. } => {
                    let key_input = KeyboardInput(input.state, input.scancode, input.virtual_keycode);
                    event!(Target::Ui, key_input);
                }
                glutin::WindowEvent::ReceivedCharacter(char) => {
                    event!(Target::Ui, ReceivedCharacter(char));
                }
                _ => (),
            }
        });
    }
}

pub struct EscKeyCloseHandler;
impl UiEventHandler<KeyboardInput> for EscKeyCloseHandler {
    fn handle(&mut self, event: &KeyboardInput, ui: &mut Ui) {
        if let KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape)) = *event {
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
use webrender::renderer::PROFILER_DBG;
impl UiEventHandler<KeyboardInput> for DebugSettingsHandler {
    fn handle(&mut self, event: &KeyboardInput, ui: &mut Ui) {
        if let KeyboardInput(ElementState::Released, _, Some(glutin::VirtualKeyCode::F1)) = *event {
            self.debug_on = !self.debug_on;
            ui.set_debug_draw_bounds(self.debug_on);
        }
        if let KeyboardInput(ElementState::Released, _, Some(glutin::VirtualKeyCode::F2)) = *event {
            ui.debug_constraints();
        }
        if let KeyboardInput(ElementState::Released, _, Some(glutin::VirtualKeyCode::F3)) = *event {
            ui.debug_widget_positions();
        }
        if let KeyboardInput(ElementState::Released, _, Some(glutin::VirtualKeyCode::F4)) = *event {
            ui.debug_variables();
        }
        if let KeyboardInput(ElementState::Released, _, Some(glutin::VirtualKeyCode::P)) = *event {
            ui.render.toggle_flags(PROFILER_DBG);
        }
    }
}
