pub mod mouse;
pub mod keyboard;

use glutin;
use glutin::ElementState;
use webrender;

use event::{EventHandler, EventArgs};
use input::mouse::{MouseMoved, MouseButton, MouseWheel};
use input::keyboard::{KeyboardInput, ReceivedCharacter};
use geometry::Point;
use app::App;

#[derive(Clone)]
pub struct InputEvent(pub glutin::WindowEvent);

impl App {
    pub fn add_input_handlers(&mut self) {
        self.add_handler(|event: &InputEvent, args: EventArgs| {
            let InputEvent(event) = event.clone();
            match event {
                glutin::WindowEvent::Closed => {
                    args.ui.close();
                }
                glutin::WindowEvent::MouseWheel { delta, .. } => {
                    args.widget.event(MouseWheel(delta));
                }
                glutin::WindowEvent::MouseInput { state, button, .. } => {
                    args.widget.event(MouseButton(state, button));
                }
                glutin::WindowEvent::MouseMoved { position, .. } => {
                    let point = Point::new(position.0 as f32, position.1 as f32);
                    args.widget.event(MouseMoved(point));
                }
                glutin::WindowEvent::KeyboardInput { input, .. } => {
                    let key_input = KeyboardInput(input);
                    args.widget.event(key_input);
                }
                glutin::WindowEvent::ReceivedCharacter(char) => {
                    args.widget.event(ReceivedCharacter(char));
                }
                _ => (),
            }
        });
    }
}

#[derive(Debug, Copy, Clone)]
pub struct EscKeyCloseHandler;

impl EventHandler<KeyboardInput> for EscKeyCloseHandler {
    fn handle(&mut self, event: &KeyboardInput, args: EventArgs) {
        if let Some(glutin::VirtualKeyCode::Escape) = event.0.virtual_keycode {
            args.ui.close();
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct DebugSettingsHandler {
    debug_on: bool
}

impl Default for DebugSettingsHandler {
    fn default() -> Self {
        DebugSettingsHandler {
            debug_on: false,
        }
    }
}

impl DebugSettingsHandler {
    pub fn new() -> Self {
        Self::default()
    }
}

impl EventHandler<KeyboardInput> for DebugSettingsHandler {
    fn handle(&mut self, event: &KeyboardInput, args: EventArgs) {
        let ui = args.ui;
        let &KeyboardInput(input) = event;
        if ElementState::Released == input.state {
            match input.virtual_keycode {
                Some(glutin::VirtualKeyCode::F1) => {
                    self.debug_on = !self.debug_on;
                    ui.set_debug_draw_bounds(self.debug_on);
                },
                Some(glutin::VirtualKeyCode::F2) => ui.solver.debug_constraints(),
                Some(glutin::VirtualKeyCode::F3) => ui.debug_widget_positions(),
                Some(glutin::VirtualKeyCode::F4) => ui.solver.debug_variables(),
                Some(glutin::VirtualKeyCode::P) => ui.render.toggle_flags(webrender::PROFILER_DBG),
                _ => {}
            }

        }
    }
}
