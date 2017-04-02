extern crate find_folder;
extern crate graphics;
extern crate backend;
extern crate glutin;

use self::backend::Window;
use limn::app::App;
use limn::input::{EscKeyCloseHandler, DebugSettingsHandler};
use limn::resources::{FontId, ImageId, resources};
use limn::widget::WidgetBuilder;

/// Create the window and initialize the app.
/// The window size is initialized to 100x100 and then resized later based
/// on the measured UI size.
/// Ideally the window wouldn't be created until the UI size is known, but
/// the window is needed right now to have a GL context for creating
/// and measuring images/text.
pub fn init_default(title: &str) -> (Window, App) {
    let window_dims = (100, 100);
    let mut window = Window::new(title, window_dims, Some(window_dims));
    let app = App::new(&mut window);
    (window, app)
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

pub fn set_root_and_loop(mut window: Window,
                         mut app: App,
                         root_widget: WidgetBuilder)
{
    app.ui.set_root(root_widget);
    app.resize_window_to_fit(&window);

    // Closes app on ESC key
    app.add_handler(EscKeyCloseHandler);
    // Toggles debug bounds drawing on F1 key
    app.add_handler(DebugSettingsHandler::new());
    app.main_loop(&mut window);
}
