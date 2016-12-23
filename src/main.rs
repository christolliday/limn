extern crate backend;
extern crate graphics;
extern crate cassowary;
extern crate input;
extern crate window;
extern crate petgraph;
extern crate find_folder;
extern crate rusttype;

#[macro_use]
extern crate matches;

pub mod widget;
pub mod ui;
pub mod util;
pub mod text;

use ui::*;
use util::*;
use widget::text::*;

use widget::{Widget, EventListener};
use widget::primitives::{RectDrawable, EllipseDrawable};

use input::{ResizeEvent, MouseCursorEvent, Event, Input};
use backend::{Window, WindowEvents, OpenGL};
use graphics::clear;

use cassowary::WeightedRelation::*;
use cassowary::strength::*;

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

    let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
    let font_path = assets.join("fonts/Hack/Hack-Regular.ttf");



    let circle2 = EllipseDrawable { background: [1.0, 1.0, 1.0, 1.0] };
    let box3 = Widget::new(Box::new(circle2));
    let rect = RectDrawable { background: [1.0, 0.0, 0.0, 1.0] };
    let mut box1 = Widget::new(Box::new(rect));
    let circle = EllipseDrawable { background: [1.0, 0.0, 1.0, 1.0] };

    struct ClickListener {}
    impl EventListener for ClickListener {
        fn matches(&self, event: &Event) -> bool {
            matches!(event, &Event::Input(Input::Move(_)))
        }
        fn handle_event(&self, event: &Event) {
            println!("event {:?}", event);
        }
    }
    let listener = ClickListener {};
    let mut box2 = Widget::new(Box::new(circle));
    box2.listeners.push(Box::new(listener));

    let ui = &mut Ui::new(&mut window, window_dim);

    let font_id = ui.resources.fonts.insert_from_file(font_path).unwrap();

    let text_drawable = TextDrawable { text: "HELLO".to_owned(), font_id: font_id, font_size: 40.0, text_color: [0.0,0.0,0.0,1.0], background_color: [1.0,1.0,1.0,1.0] };
    let mut text_widget = Widget::new(Box::new(text_drawable));
    let text_constraints = [text_widget.layout.top | EQ(REQUIRED) | 100.0,
                            text_widget.layout.left | EQ(REQUIRED) | 100.0];
    text_widget.layout.width(300.0, WEAK);
    text_widget.layout.height(50.0, WEAK);
    text_widget.layout.add_constraints(&text_constraints);

    let box1_constraints = [box1.layout.top | EQ(REQUIRED) | 0.0,
                            box1.layout.left | EQ(REQUIRED) | 0.0,
                            box1.layout.left | LE(REQUIRED) | box1.layout.right];
    box1.layout.width(50.0, WEAK);
    box1.layout.height(100.0, WEAK);
    box1.layout.add_constraints(&box1_constraints);

    let box2_constraints = [box2.layout.bottom | EQ(REQUIRED) | ui.window_height, // bottom align
                            box2.layout.right | EQ(REQUIRED) | ui.window_width, // right align
                            box2.layout.left | GE(REQUIRED) | box1.layout.right, // no overlap
                            box2.layout.left | LE(REQUIRED) | box2.layout.right];
    box2.layout.width(100.0, WEAK);
    box2.layout.height(100.0, WEAK);
    box2.layout.add_constraints(&box2_constraints);

    let root_index = ui.root;
    let box1_index = ui.add_widget(root_index, box1);
    ui.add_widget(root_index, box2);
    ui.add_widget(box1_index, box3);
    ui.add_widget(root_index, text_widget);
    ui.init();

    // Poll events from the window.
    while let Some(event) = events.next(&mut window) {
        window.handle_event(&event);
        if let Some(window_dims) = event.resize_args() {
            ui.resize_window(window_dims);
        }
        if let Some(_) = event.mouse_cursor_args() {
            ui.post_event(&event);
        }

        window.draw_2d(&event, |c, g| {
            clear([0.8, 0.8, 0.8, 1.0], g);
            ui.draw(c, g);
        });
    }
}
