extern crate find_folder;
extern crate graphics;
extern crate backend;
extern crate glutin;

use self::glutin::Event;
use self::backend::{Window, WindowEvents};
use self::backend::events::WindowEvent;
use limn::ui::Ui;
use limn::resources::{FontId, ImageId, resources};
use limn::util::Dimensions;
use limn::widget::builder::WidgetBuilder;

pub fn init_default(title: &str) -> (Window, Ui) {
    let window_dims = (100, 100);
    let mut window = Window::new(title, window_dims, Some(window_dims));
    let ui = Ui::new(&mut window);
    (window, ui)
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
                         mut ui: Ui,
                         root_widget: WidgetBuilder) {
    ui.graph.set_root(root_widget);
    ui.graph.resize_window_to_fit(&window);

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
                        ui.graph.window_resized(Dimensions {
                            width: width as f64,
                            height: height as f64,
                        });
                        ui.graph.update_layout();
                    }
                    _ => (),
                }
                ui.graph.handle_input(event.clone(), &mut ui.event_queue);
                ui.event_queue.handle_events(&mut ui.graph, &mut ui.event_handlers);
            }
            WindowEvent::Render => {
                if ui.graph.dirty_widgets.len() > 0 {
                    window.draw_2d(|context, graphics| {
                        graphics::clear([0.8, 0.8, 0.8, 1.0], graphics);
                        ui.graph.draw(context, graphics);
                    });
                    ui.graph.dirty_widgets.clear();
                }
            }
        }
    }
}
