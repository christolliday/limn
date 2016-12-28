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

use limn::widget::{self, Widget, EventHandler};
use limn::widget::image::ImageDrawable;
use limn::widget::primitives::{RectDrawable, EllipseDrawable};

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

    let (box1, box2, box3, text_widget, image_widget) = {
        
        let ref root = ui.graph[ui.root_index];

        let circle2 = EllipseDrawable { background: [1.0, 1.0, 1.0, 1.0] };
        let box3 = Widget::new(widget::primitives::draw_ellipse, Box::new(circle2));

        let rect = RectDrawable { background: [1.0, 0.0, 0.0, 1.0] };
        let mut box1 = Widget::new(widget::primitives::draw_rect, Box::new(rect));

        let circle = EllipseDrawable { background: [1.0, 0.0, 1.0, 1.0] };
        let mut box2 = Widget::new(widget::primitives::draw_ellipse, Box::new(circle));

        let text_drawable = TextDrawable { text: "HELLO".to_owned(), font_id: font_id, font_size: 40.0, text_color: [0.0,0.0,0.0,1.0], background_color: [1.0,1.0,1.0,1.0] };
        let text_dims = text_drawable.measure_dims_no_wrap(&ui.resources);
        let mut text_widget = Widget::new(widget::text::draw_text, Box::new(text_drawable));
        let text_constraints = [text_widget.layout.top | EQ(REQUIRED) | 100.0,
                                text_widget.layout.left | EQ(REQUIRED) | 100.0];
        text_widget.layout.width(text_dims.width);
        text_widget.layout.height(text_dims.height);
        text_widget.layout.add_constraints(&text_constraints);

        let box1_constraints = [box1.layout.top | EQ(REQUIRED) | 0.0,
                                box1.layout.left | EQ(REQUIRED) | 0.0,
                                box1.layout.left | LE(REQUIRED) | box1.layout.right];
        box1.layout.width(50.0);
        box1.layout.height(100.0);
        box1.layout.add_constraints(&box1_constraints);

        let box2_constraints = [box2.layout.bottom | EQ(REQUIRED) | root.layout.bottom, // bottom align
                                box2.layout.right | EQ(REQUIRED) | root.layout.right, // right align
                                box2.layout.left | GE(REQUIRED) | box1.layout.right, // no overlap
                                box2.layout.left | LE(REQUIRED) | box2.layout.right];
        box2.layout.width(100.0);
        box2.layout.height(100.0);
        box2.layout.add_constraints(&box2_constraints);

        let image_drawable = ImageDrawable { image_id: image_id };
        let image_dims = image_drawable.measure_image(&ui.resources);
        let mut image_widget = Widget::new(widget::image::draw_image, Box::new(image_drawable));
        image_widget.layout.width(image_dims.width);
        image_widget.layout.height(image_dims.height);
        image_widget.layout.center(&root.layout);

        (box1, box2, box3, text_widget, image_widget)
    };

    let root_index = ui.root_index;
    let box1_index = ui.add_widget(root_index, box1);
    ui.add_widget(root_index, box2);
    ui.add_widget(box1_index, box3);
    ui.add_widget(root_index, text_widget);
    ui.add_widget(root_index, image_widget);
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
