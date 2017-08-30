extern crate find_folder;
extern crate glutin;
extern crate env_logger;
extern crate log;

use limn::window::Window;
use limn::app::App;
use limn::input::{EscKeyCloseHandler, DebugSettingsHandler};
use limn::util::Size;

/// Create the window and initialize the app.
/// The window size is initialized to 100x100 and then resized later based
/// on the measured UI size.
/// Ideally the window wouldn't be created until the UI size is known, but
/// the window is needed right now to have a GL context for creating
/// and measuring images/text.
#[allow(dead_code)]
pub fn init_default(title: &str) -> App {
    init(title, None)
}

#[allow(dead_code)]
pub fn init_default_min_size(title: &str, size: Size) -> App {
    init(title, Some((size.width as u32, size.height as u32)))
}

fn init(title: &str, size: Option<(u32, u32)>) -> App {
    env_logger::init().unwrap();
    let window_size = size.unwrap_or((100, 100));
    let events_loop = glutin::EventsLoop::new();
    let window = Window::new(title, window_size, Some(window_size), &events_loop);
    let mut app = App::new(window, events_loop);

    // Closes app on ESC key
    app.add_handler(EscKeyCloseHandler);
    // Toggles debug bounds drawing on F1 key
    app.add_handler(DebugSettingsHandler::new());
    app
}
