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
use limn::widget::layout::{LinearLayout, Orientation};

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
    
    let (scroll_widget, list_widget, list_item_widgets) = {
        let ref root = ui.graph[ui.root_index];

        let mut scroll_widget = Widget::new();
        let constraints = &[
            scroll_widget.layout.left | EQ(REQUIRED) | 50.0,
            scroll_widget.layout.right | EQ(REQUIRED) | root.layout.right - 50.0,
            scroll_widget.layout.top | EQ(REQUIRED) | 50.0,
            scroll_widget.layout.bottom | EQ(REQUIRED) | root.layout.bottom - 50.0,
        ];
        scroll_widget.layout.add_constraints(constraints);
        scroll_widget.layout.scrollable = true;
        scroll_widget.event_handlers.push(Box::new(ScrollHandler::new()));

        let mut list_widget = Widget::new();
        list_widget.layout.match_width(&scroll_widget.layout);
        list_widget.event_handlers.push(Box::new(WidgetScrollHandler::new()));

        let list_item_widgets = {
            let mut linear_layout = LinearLayout::new(Orientation::Vertical, &list_widget.layout);
            let mut list_item_widgets = Vec::new();
            for i in 1..15 {
                let mut list_item_widget = Widget::new();
                let text_drawable = TextDrawable::new("hello".to_owned(), font_id);
                let text_dims = text_drawable.measure_dims_no_wrap(&ui.resources);
                list_item_widget.set_drawable(widget::text::draw_text, Box::new(text_drawable));
                list_item_widget.layout.height_strength(text_dims.height, STRONG);
                linear_layout.add_widget(&mut list_item_widget);
                list_item_widgets.push(list_item_widget);
            }
            list_item_widgets
        };
        (scroll_widget, list_widget, list_item_widgets)
    };

    let root_index = ui.root_index;
    let scroll_index = ui.add_widget(root_index, scroll_widget);
    let list_index = ui.add_widget(scroll_index, list_widget);
    for list_item_widget in list_item_widgets {
        ui.add_widget(list_index, list_item_widget);
    }

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
