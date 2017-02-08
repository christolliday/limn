extern crate find_folder;
extern crate graphics;
extern crate backend;
extern crate glutin;

use self::glutin::Event;
use self::backend::{Window, WindowEvents};
use self::backend::events::WindowEvent;
use limn::ui::{self, Ui, UiEventArgs, UiEventHandler};
use limn::resources::{FontId, ImageId, resources};
use limn::util::Dimensions;
use limn::widget::builder::WidgetBuilder;
use limn::event::EventQueue;

pub fn init_default(title: &str) -> (Window, Ui, EventQueue) {
    let window_dims = (100, 100);
    let mut window = Window::new(title, window_dims, Some(window_dims));
    let mut event_queue = EventQueue::new(&window);
    let ui = Ui::new(&mut window, &event_queue);

    (window, ui, event_queue)
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

pub fn set_root_and_loop(mut window: Window, mut ui: Ui, root_widget: WidgetBuilder, mut event_queue: EventQueue, mut ui_event_handlers: Vec<Box<UiEventHandler>>) {
    ui.set_root(root_widget);
    ui.resize_window_to_fit(&window);

    ui_event_handlers.append(&mut ui::get_default_event_handlers());
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
                        println!("{:?} {:?}", width, height);
                        ui.window_resized(Dimensions {
                            width: width as f64,
                            height: height as f64,
                        });
                        ui.update_layout();
                    }
                    _ => (),
                }
                ui.handle_input(event.clone(), &mut event_queue);
                event_queue.handle_events(&mut ui, &mut ui_event_handlers);
            }
            WindowEvent::Render => {
                if ui.dirty_widgets.len() > 0 {
                    window.draw_2d(|context, graphics| {
                        graphics::clear([0.8, 0.8, 0.8, 1.0], graphics);
                        ui.draw(context, graphics);
                    });
                    ui.dirty_widgets.clear();
                }
            }
        }
    }
}
