extern crate env_logger;

use limn::prelude::*;
use limn::input::{EscKeyCloseHandler, DebugSettingsHandler};

// Initialize a limn App with common handlers and set up logger
pub fn init(window_builder: glutin::WindowBuilder) -> App {
    env_logger::init().unwrap();
    let events_loop = glutin::EventsLoop::new();
    let window = Window::new(window_builder, &events_loop);
    let mut app = App::new(window, events_loop);

    // Closes app on ESC key
    app.add_handler(EscKeyCloseHandler);
    // Toggles debug bounds drawing on F1 key
    app.add_handler(DebugSettingsHandler::new());
    app
}
