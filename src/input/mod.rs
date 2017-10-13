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
        self.add_handler_fn(|event: &InputEvent, args| {
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
                    let key_input = KeyboardInput(input.state, input.scancode, input.virtual_keycode);
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
        if let KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape)) = *event {
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
        if let KeyboardInput(ElementState::Released, _, Some(glutin::VirtualKeyCode::F1)) = *event {
            self.debug_on = !self.debug_on;
            ui.set_debug_draw_bounds(self.debug_on);
        }
        if let KeyboardInput(ElementState::Released, _, Some(glutin::VirtualKeyCode::F2)) = *event {
            ui.solver.debug_constraints();
        }
        if let KeyboardInput(ElementState::Released, _, Some(glutin::VirtualKeyCode::F3)) = *event {
            ui.debug_widget_positions();
        }
        if let KeyboardInput(ElementState::Released, _, Some(glutin::VirtualKeyCode::F4)) = *event {
            ui.solver.debug_variables();
        }
        if let KeyboardInput(ElementState::Released, _, Some(glutin::VirtualKeyCode::P)) = *event {
            ui.render.toggle_flags(webrender::PROFILER_DBG);
        }
    }
}
