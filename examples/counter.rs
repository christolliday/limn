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
use limn::widget::builder::WidgetBuilder;
use limn::widget::primitives::{RectDrawable};
use limn::widget::button::{ButtonEventHandler, ButtonOnHandler, ButtonOffHandler};
use limn::widget::layout::{LinearLayout, Orientation};

use backend::{Window, WindowEvents, OpenGL};
use input::{ResizeEvent, MouseCursorEvent, PressEvent, ReleaseEvent, Event, Input, EventId};

use std::any::Any;

fn main() {
    let window_dim = Dimensions {
        width: 200.0,
        height: 100.0,
    };

    // Construct the window.
    let mut window: Window = backend::window::WindowSettings::new("Limn Button Demo", window_dim)
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

    let font_id = ui.resources.fonts.insert_from_file(font_path).unwrap();
    
    let mut root_widget = WidgetBuilder::new();
    {
        let ref root = ui.graph[ui.root_index];
        root_widget.layout.match_layout(&root.layout);
    }
    let mut linear_layout = LinearLayout::new(Orientation::Horizontal, &root_widget.layout);

    let text_drawable = TextDrawable { text: "0".to_owned(), font_id: font_id, font_size: 20.0, text_color: [0.0,0.0,0.0,1.0], background_color: [1.0,1.0,1.0,1.0] };
    let text_dims = text_drawable.measure_dims_no_wrap(&ui.resources);
    let mut text_widget = WidgetBuilder::new();
    text_widget.set_drawable(widget::text::draw_text, Box::new(text_drawable));
    text_widget.layout.width(80.0);
    text_widget.layout.height(text_dims.height);
    text_widget.layout.center_vertical(&root_widget.layout);
    linear_layout.add_widget(&mut text_widget.layout);
    root_widget.add_child(Box::new(text_widget));

    let mut button_container = WidgetBuilder::new();
    linear_layout.add_widget(&mut button_container.layout);
    
    let rect = RectDrawable { background: [1.0, 0.0, 0.0, 1.0] };
    let mut button_widget = WidgetBuilder::new();
    button_widget.set_drawable(widget::primitives::draw_rect, Box::new(rect));
    button_widget.event_handlers.push(Box::new(ButtonEventHandler::new()));
    button_widget.event_handlers.push(Box::new(ButtonOnHandler{}));
    button_widget.event_handlers.push(Box::new(ButtonOffHandler{}));
    button_widget.debug_color([1.0, 1.0, 0.0, 1.0]);
    button_widget.layout.dimensions(Dimensions { width: 100.0, height: 50.0 });
    button_widget.layout.center(&button_container.layout);

    let button_text_drawable = TextDrawable { text: "Count".to_owned(), font_id: font_id, font_size: 20.0, text_color: [0.0,0.0,0.0,1.0], background_color: [0.0, 0.0, 0.0, 0.0] };
    let button_text_dims = button_text_drawable.measure_dims_no_wrap(&ui.resources);
    let mut button_text_widget = WidgetBuilder::new();
    button_text_widget.set_drawable(widget::text::draw_text, Box::new(button_text_drawable));
    button_text_widget.event_handlers.push(Box::new(ButtonOnHandler{}));
    button_text_widget.event_handlers.push(Box::new(ButtonOffHandler{}));
    button_text_widget.layout.dimensions(button_text_dims);
    button_text_widget.layout.center(&button_widget.layout);

    button_widget.add_child(Box::new(button_text_widget));
    button_container.add_child(Box::new(button_widget));
    root_widget.add_child(Box::new(button_container));

    let root_index = ui.root_index;
    root_widget.create(ui, Some(root_index));


    // Poll events from the window.
    while let Some(event) = events.next(&mut window) {
        window.handle_event(&event);
        if let Some(window_dims) = event.resize_args() {
            ui.resize_window(window_dims.into());
        }
        ui.handle_event(event.clone());
        window.draw_2d(&event, |c, g| {
            graphics::clear([0.8, 0.8, 0.8, 1.0], g);
            ui.draw(c, g);
        });
    }
}
