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
use limn::widget::scroll::{ScrollHandler, WidgetScrollHandler};
use limn::widget::button::{ButtonEventHandler, ButtonOnHandler, ButtonOffHandler};

use backend::{Window, WindowEvents, OpenGL};
use input::{ResizeEvent, MouseCursorEvent, PressEvent, ReleaseEvent, Event, Input, EventId};

use cassowary::WeightedRelation::*;
use cassowary::strength::*;

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
    
    let (scroll_widget, rect_container_widget, rect_tl_widget, rect_tr_widget, rect_bl_widget, rect_br_widget) = {
        let ref root = ui.graph[ui.root_index];

        let mut scroll_widget = Widget::new();
        let constraints = &[
            scroll_widget.layout.left | EQ(REQUIRED) | 100.0,
            scroll_widget.layout.top | EQ(REQUIRED) | 100.0,
        ];
        scroll_widget.layout.add_constraints(constraints);
        scroll_widget.layout.width(200.0);
        scroll_widget.layout.height(200.0);
        scroll_widget.layout.scrollable = true;
        scroll_widget.event_handlers.push(Box::new(ScrollHandler::new()));

        let mut rect_container_widget = Widget::new();
        rect_container_widget.event_handlers.push(Box::new(WidgetScrollHandler::new()));
        rect_container_widget.layout.dimensions(Dimensions { width: 400.0, height: 400.0});

        let mut rect_tl_widget = Widget::new();
        rect_tl_widget.set_drawable(widget::primitives::draw_rect, Box::new(RectDrawable { background: [1.0, 0.0, 0.0, 1.0]}));
        rect_tl_widget.layout.dimensions(Dimensions { width: 200.0, height: 200.0});
        rect_tl_widget.layout.align_top(&rect_container_widget.layout);
        rect_tl_widget.layout.align_left(&rect_container_widget.layout);

        let mut rect_tr_widget = Widget::new();
        rect_tr_widget.set_drawable(widget::primitives::draw_rect, Box::new(RectDrawable { background: [0.0, 1.0, 0.0, 1.0]}));
        rect_tr_widget.layout.dimensions(Dimensions { width: 200.0, height: 200.0});
        rect_tr_widget.layout.align_top(&rect_container_widget.layout);
        rect_tr_widget.layout.align_right(&rect_container_widget.layout);

        let mut rect_bl_widget = Widget::new();
        rect_bl_widget.set_drawable(widget::primitives::draw_rect, Box::new(RectDrawable { background: [0.0, 0.0, 1.0, 1.0]}));
        rect_bl_widget.layout.dimensions(Dimensions { width: 200.0, height: 200.0});
        rect_bl_widget.layout.align_bottom(&rect_container_widget.layout);
        rect_bl_widget.layout.align_left(&rect_container_widget.layout);

        let mut rect_br_widget = Widget::new();
        rect_br_widget.set_drawable(widget::primitives::draw_rect, Box::new(RectDrawable { background: [1.0, 0.0, 1.0, 1.0]}));
        rect_br_widget.layout.dimensions(Dimensions { width: 200.0, height: 200.0});
        rect_br_widget.layout.align_bottom(&rect_container_widget.layout);
        rect_br_widget.layout.align_right(&rect_container_widget.layout);


        (scroll_widget, rect_container_widget, rect_tl_widget, rect_tr_widget, rect_bl_widget, rect_br_widget)
    };

    let root_index = ui.root_index;
    let scroll_index = ui.add_widget(root_index, scroll_widget);
    let rect_container_index = ui.add_widget(scroll_index, rect_container_widget);
    ui.add_widget(rect_container_index, rect_tl_widget);
    ui.add_widget(rect_container_index, rect_tr_widget);
    ui.add_widget(rect_container_index, rect_bl_widget);
    ui.add_widget(rect_container_index, rect_br_widget);

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
