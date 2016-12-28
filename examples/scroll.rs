extern crate limn;
extern crate backend;
extern crate cassowary;
extern crate graphics;
extern crate input;
extern crate window;
extern crate find_folder;

#[macro_use]
extern crate matches;

use limn::ui::*;
use limn::util::*;
use limn::widget::text::*;
use limn::widget;
use limn::event;

use limn::widget::{Widget, EventHandler};
use limn::widget::primitives::{RectDrawable};
use limn::widget::image::ImageDrawable;
use limn::widget::button::{ButtonEventHandler, ButtonOnHandler, ButtonOffHandler};

use backend::{Window, WindowEvents, OpenGL};
use input::{ResizeEvent, MouseCursorEvent, PressEvent, ReleaseEvent, Event, Input, EventId};

use std::any::Any;

fn main() {
    let window_dim = Dimensions {
        width: 720.0,
        height: 400.0,
    };

    // Construct the window.
    let mut window: Window = backend::window::WindowSettings::new("Limn Scroll Demo", window_dim)
        .opengl(OpenGL::V3_2)
        .samples(4)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create the event loop.
    let mut events = WindowEvents::new();

    let ui = &mut Ui::new(&mut window, window_dim);

    let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
    let font_path = assets.join("fonts/Hack/Hack-Regular.ttf");
    let image_path = assets.join("images/rust.png");

    let font_id = ui.resources.fonts.insert_from_file(font_path).unwrap();
    let image_id = ui.resources.images.insert_from_file(&mut window.context.factory, image_path);
    
    let image_widget = {
        let ref root = ui.graph[ui.root_index];

        let image_drawable = ImageDrawable { image_id: image_id };
        let image_dims = image_drawable.measure_image(&ui.resources);
        let mut image_widget = Widget::new();
        image_widget.set_drawable(widget::image::draw_image, Box::new(image_drawable));
        image_widget.layout.width(image_dims.width * 2.0);
        image_widget.layout.height(image_dims.height * 2.0);
        image_widget.layout.center(&root.layout);
        image_widget
    };

    let root_index = ui.root_index;
    ui.add_widget(root_index, image_widget);
    ui.init();

    // Poll events from the window.
    while let Some(event) = events.next(&mut window) {
        window.handle_event(&event);
        if let Some(window_dims) = event.resize_args() {
            ui.resize_window(window_dims.into());
        }
        ui.handle_event(&event);
        window.draw_2d(&event, |c, g| {
            graphics::clear([0.8, 0.8, 0.8, 1.0], g);
            ui.draw(c, g);
        });
    }
}
