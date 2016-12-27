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

use limn::widget::{Widget, EventListener};
use limn::widget::image::ImageDrawable;
use limn::widget::primitives::{RectDrawable, EllipseDrawable};

use backend::{Window, WindowEvents, OpenGL};
use input::{ResizeEvent, MouseCursorEvent, Event, Input};

use cassowary::WeightedRelation::*;
use cassowary::strength::*;

use std::any::Any;

fn main() {
    let window_dim = Dimensions {
        width: 720.0,
        height: 400.0,
    };

    // Construct the window.
    let mut window: Window = backend::window::WindowSettings::new("Grafiki Demo", window_dim)
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
    
    let (button_widget, text_widget) = {
        let ref root = ui.graph[ui.root_index];

        struct ClickListener {
            on: bool,
        }
        impl ClickListener {
            fn new() -> Self {
                ClickListener { on: false }
            }
        }
        impl EventListener for ClickListener {
            fn matches(&self, event: &Event) -> bool {
                matches!(event, &Event::Input(Input::Move(_)))
            }
            fn handle_event(&mut self, state: &mut Any, event: &Event) {
                self.on = !self.on;
                let drawable: &mut RectDrawable = state.downcast_mut().unwrap();
                drawable.background = if self.on { [0.0, 0.0, 0.0, 1.0] } else { [1.0, 0.0, 0.0, 1.0] };
            }
        }

        let rect = RectDrawable { background: [1.0, 0.0, 0.0, 1.0] };
        let mut button_widget = Widget::new(widget::primitives::draw_rect, Box::new(rect));
        let listener = ClickListener::new();
        button_widget.listeners.push(Box::new(listener));
        button_widget.layout.width(300.0, STRONG);
        button_widget.layout.height(100.0, STRONG);
        button_widget.layout.center(&root.layout);

        let text_drawable = TextDrawable { text: "ON".to_owned(), font_id: font_id, font_size: 40.0, text_color: [0.0,0.0,0.0,1.0], background_color: [1.0,1.0,1.0,1.0] };
        let text_dims = text_drawable.measure_dims_no_wrap(&ui.resources);
        let mut text_widget = Widget::new(widget::text::draw_text, Box::new(text_drawable));
        text_widget.layout.width(text_dims.width, STRONG);
        text_widget.layout.height(text_dims.height, STRONG);
        text_widget.layout.center(&button_widget.layout);
        (button_widget, text_widget)
    };

    let root_index = ui.root_index;
    let button_index = ui.add_widget(root_index, button_widget);
    ui.add_widget(button_index, text_widget);
    ui.init();

    // Poll events from the window.
    while let Some(event) = events.next(&mut window) {
        window.handle_event(&event);
        if let Some(window_dims) = event.resize_args() {
            ui.resize_window(window_dims.into());
        }
        if let Some(_) = event.mouse_cursor_args() {
            ui.post_event(&event);
        }
        window.draw_2d(&event, |c, g| {
            graphics::clear([0.8, 0.8, 0.8, 1.0], g);
            ui.draw(c, g);
        });
    }
}
