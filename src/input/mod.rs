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
pub struct InputHandler;
impl ui::EventHandler<InputEvent> for InputHandler {
    fn handle(&mut self, event: &InputEvent, args: ui::EventArgs) {
        let queue = args.queue;
        let InputEvent(event) = event.clone();
        match event {
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
}