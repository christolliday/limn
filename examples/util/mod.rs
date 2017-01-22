extern crate find_folder;
extern crate graphics;
extern crate input;
extern crate backend;

use self::input::ResizeEvent;
use self::backend::{Window, WindowEvents};
use self::backend::events::WindowEvent;
use limn::ui::Ui;
use limn::resources::Id;
use limn::util::Dimensions;
use limn::widget::builder::WidgetBuilder;

pub fn init_default(title: &str) -> (Window, Ui) {
    let window_dims = Dimensions { width: 100.0, height: 100.0 };
    let mut window = Window::new(title, window_dims, Some(window_dims));
    let ui = Ui::new(&mut window);

    (window, ui)
}

pub fn load_default_font(ui: &mut Ui) -> Id {
    let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
    let font_path = assets.join("fonts/Hack/Hack-Regular.ttf");
    ui.resources.fonts.insert_from_file(font_path).unwrap()
}

pub fn load_default_image(ui: &mut Ui, window: &mut Window) -> Id {
    let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
    let image_path = assets.join("images/rust.png");
    ui.resources.images.insert_from_file(&mut window.context.factory, image_path)
}

pub fn set_root_and_loop(mut window: Window, mut ui: Ui, root_widget: WidgetBuilder) {
    ui.set_root(root_widget);
    ui.resize_window_to_fit(&window);
    let mut events = WindowEvents::new();
    while let Some(event) = events.next(&mut window) {
        match event {
            WindowEvent::Input(event) => {
                if let Some(window_dims) = event.resize_args() {
                    window.window_resized();
                    ui.window_resized(&mut window, window_dims.into());
                }
                ui.handle_event(event.clone());
            },
            WindowEvent::Render => {
                window.draw_2d(|context, graphics| {
                    graphics::clear([0.8, 0.8, 0.8, 1.0], graphics);
                    ui.draw(context, graphics);
                });
            }
        }
    }
}