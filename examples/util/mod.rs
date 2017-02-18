extern crate find_folder;
extern crate graphics;
extern crate backend;
extern crate glutin;

use self::glutin::Event;
use self::backend::{Window, WindowEvents};
use self::backend::events::WindowEvent;
use limn::app::App;
use limn::ui::{Ui, InputEvent};
use limn::ui::queue::EventAddress;
use limn::resources::{FontId, ImageId, resources};
use limn::util::Dimensions;
use limn::widget::builder::WidgetBuilder;

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

    app.ui.graph.set_root(root_widget, &mut app.ui.solver);
    app.ui.graph.resize_window_to_fit(&window, &mut app.ui.solver);

    let mut events = WindowEvents::new();
    while let Some(event) = events.next(&mut window.window) {
        match event {
            WindowEvent::Input(event) => {
                match event {
                    Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape)) |
                    Event::Closed => {
                        break;
                    }
                    Event::Resized(width, height) => {
                        window.window_resized();
                        app.ui.graph.window_resized(Dimensions {
                            width: width as f64,
                            height: height as f64,
                        }, &mut app.ui.solver);
                        app.ui.graph.update_layout(&mut app.ui.solver);
                    }
                    Event::Awakened => {}
                    _ => {
                        app.event_queue.push(EventAddress::Ui, InputEvent(event));
                        app.handle_events();
                    },
                }
            }
            WindowEvent::Render => {
                if app.ui.graph.dirty_widgets.len() > 0 {
                    window.draw_2d(|context, graphics| {
                        graphics::clear([0.8, 0.8, 0.8, 1.0], graphics);
                        app.ui.graph.draw(context, graphics);
                    });
                    app.ui.graph.dirty_widgets.clear();
                }
            }
        }
    }
}
