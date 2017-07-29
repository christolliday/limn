extern crate find_folder;
extern crate graphics;
extern crate backend;
extern crate glutin;
extern crate env_logger;
extern crate log;

use self::backend::Window;
use limn::app::App;
use limn::input::{EscKeyCloseHandler, DebugSettingsHandler};
use limn::resources::{FontId, ImageId, resources};
use limn::widget::WidgetBuilder;
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
    let app = App::new(window, events_loop);
    app
}

#[allow(dead_code)]
pub fn load_default_font() -> FontId {
    let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
    let font_path = assets.join("fonts/Hack/Hack-Regular.ttf");
    let mut res = resources();
    res.fonts.insert_from_file(font_path).unwrap()
}

#[allow(dead_code)]
pub fn load_default_image(window: &mut Window) -> ImageId {
    let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
    let image_path = assets.join("images/rust.png");
    resources().images.insert_from_file(&mut window.context.factory, image_path)
}

pub fn set_root_and_loop(mut app: App, root_widget: WidgetBuilder)
{
    app.ui.root.add_child(root_widget.widget);

    // Closes app on ESC key
    app.add_handler(EscKeyCloseHandler);
    // Toggles debug bounds drawing on F1 key
    app.add_handler(DebugSettingsHandler::new());
    app.main_loop();
}
