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
use limn::util;
use limn::widget::text::*;

use limn::widget::{self, Widget, EventHandler};
use limn::widget::image::ImageDrawable;
use limn::widget::primitives::{RectDrawable, EllipseDrawable};
use limn::widget::button::{ButtonEventHandler, ButtonOnHandler, ButtonOffHandler};

use backend::{Window, WindowEvents, OpenGL};
use input::{ResizeEvent, MouseCursorEvent, Event, Input};

use cassowary::WeightedRelation::*;
use cassowary::strength::*;

fn main() {
    let window_dim = Dimensions {
        width: 720.0,
        height: 400.0,
    };

    // Construct the window.
    let mut window: Window = backend::window::WindowSettings::new("Limn Demo", window_dim)
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

    let (box_widget, ellipse_widget, inner_ellipse_widget, text_widget, image_widget) = {
        
        let ref root = ui.graph[ui.root_index];

        let mut inner_ellipse_widget = Widget::new();
        inner_ellipse_widget.set_drawable(widget::primitives::draw_ellipse, Box::new(EllipseDrawable { background: [1.0, 1.0, 1.0, 1.0] }));

        let rect = RectDrawable { background: [1.0, 0.0, 0.0, 1.0] };
        let mut box_widget = Widget::new();
        box_widget.set_drawable(widget::primitives::draw_rect, Box::new(rect));

        let circle = EllipseDrawable { background: [1.0, 0.0, 1.0, 1.0] };
        let mut ellipse_widget = Widget::new();
        ellipse_widget.set_drawable(widget::primitives::draw_ellipse, Box::new(circle));
        ellipse_widget.set_mouse_over_fn(util::mouse_inside_ellipse);
        ellipse_widget.event_handlers.push(Box::new(ButtonEventHandler::new()));
        ellipse_widget.event_handlers.push(Box::new(ButtonOnHandler{}));
        ellipse_widget.event_handlers.push(Box::new(ButtonOffHandler{}));

        let text_drawable = TextDrawable { text: "HELLO".to_owned(), font_id: font_id, font_size: 40.0, text_color: [0.0,0.0,0.0,1.0], background_color: [1.0,1.0,1.0,1.0] };
        let text_dims = text_drawable.measure_dims_no_wrap(&ui.resources);
        let mut text_widget = Widget::new();
        text_widget.set_drawable(widget::text::draw_text, Box::new(text_drawable));
        let text_constraints = [text_widget.layout.top | EQ(REQUIRED) | 100.0,
                                text_widget.layout.left | EQ(REQUIRED) | 100.0];
        text_widget.layout.dimensions(text_dims);
        text_widget.layout.add_constraints(&text_constraints);

        let box_constraints = [box_widget.layout.top | EQ(REQUIRED) | 0.0,
                               box_widget.layout.left | EQ(REQUIRED) | 0.0,
                               box_widget.layout.left | LE(REQUIRED) | box_widget.layout.right];
        box_widget.layout.dimensions(Dimensions { width: 50.0, height: 100.0 });
        box_widget.layout.add_constraints(&box_constraints);

        let ellipse_constraints = [ellipse_widget.layout.bottom | EQ(REQUIRED) | root.layout.bottom, // bottom align
                                   ellipse_widget.layout.right | EQ(REQUIRED) | root.layout.right, // right align
                                   ellipse_widget.layout.left | GE(REQUIRED) | box_widget.layout.right,]; // no overlap

        ellipse_widget.layout.dimensions(Dimensions { width: 100.0, height: 100.0 });
        ellipse_widget.layout.add_constraints(&ellipse_constraints);

        let image_drawable = ImageDrawable::new(image_id);
        let image_dims = image_drawable.measure_image(&ui.resources);
        let mut image_widget = Widget::new();
        image_widget.set_drawable(widget::image::draw_image, Box::new(image_drawable));
        image_widget.layout.width(image_dims.width);
        image_widget.layout.height(image_dims.height);
        image_widget.layout.center(&root.layout);

        (box_widget, ellipse_widget, inner_ellipse_widget, text_widget, image_widget)
    };

    let root_index = ui.root_index;
    let box_index = ui.add_widget(root_index, box_widget);
    ui.add_widget(root_index, ellipse_widget);
    ui.add_widget(box_index, inner_ellipse_widget);
    ui.add_widget(root_index, text_widget);
    ui.add_widget(root_index, image_widget);

    // Poll events from the window.
    while let Some(event) = events.next(&mut window) {
        window.handle_event(&event);
        if let Some(window_dims) = event.resize_args() {
            ui.resize_window(window_dims.into());
        }
        ui.handle_event(event.clone());
        window.draw_2d(&event, |context, graphics| {
            graphics::clear([0.8, 0.8, 0.8, 1.0], graphics);
            ui.draw(context, graphics);
        });
    }
}
