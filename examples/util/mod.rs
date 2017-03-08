extern crate find_folder;
extern crate graphics;
extern crate backend;
extern crate glutin;

use self::glutin::Event;
use self::backend::{Window, WindowEvents};
use self::backend::events::WindowEvent;
use limn::app::App;
use limn::input::{InputEvent, EscKeyCloseHandler};
use limn::event::Target;
use limn::resources::{FontId, ImageId, resources};
use limn::util::Dimensions;
use limn::widget::WidgetBuilder;

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
                         root_widget: WidgetBuilder) {

    app.ui.set_root(root_widget);
    // handle layout change events, needed to measure widgets before resizing window
    app.handle_events();
    app.ui.graph.resize_window_to_fit(&window);

    app.add_handler(EscKeyCloseHandler);
    let mut events = WindowEvents::new();
    while !app.ui.should_close() {
        let event = events.next(&mut window.window);
        match event {
            WindowEvent::Input(event) => {
                match event {
                    Event::Resized(width, height) => {
                        window.window_resized();
                        app.ui.graph.window_resized(Dimensions {
                            width: width as f64,
                            height: height as f64,
                        }, &mut app.ui.solver);
                        events.update();
                    }
                    Event::Awakened => {
                        app.handle_events();
                    }
                    _ => {
                        app.queue.push(Target::Ui, InputEvent(event));
                        app.handle_events();
                    },
                }
            }
            WindowEvent::Render => {
                app.render(&mut window);
            }
        }
    }
}
