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
use limn::widget::image::ImageDrawable;
use limn::widget::scroll::{ScrollHandler, WidgetScrollHandler};
use limn::widget::button::{ButtonEventHandler, ButtonOnHandler, ButtonOffHandler};
use limn::widget::layout::{LinearLayout, Orientation};

use backend::{Window, WindowEvents, OpenGL};
use backend::glyph::GlyphCache;
use backend::events::WindowEvent;
use input::{ResizeEvent, MouseCursorEvent, PressEvent, ReleaseEvent, Event, Input, EventId};

use cassowary::WeightedRelation::*;
use cassowary::strength::*;

use std::any::Any;

fn main() {
    let window_dims = Dimensions { width: 100.0, height: 100.0 };

    let mut window = Window::new("Limn list demo", window_dims, None);
    let mut resources = Resources::new();

    let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
    let font_path = assets.join("fonts/Hack/Hack-Regular.ttf");
    let image_path = assets.join("images/rust.png");

    let font_id = resources.fonts.insert_from_file(font_path).unwrap();
    let image_id = resources.images.insert_from_file(&mut window.context.factory, image_path);
    
    let mut root_widget = WidgetBuilder::new();

    let mut scroll_widget = WidgetBuilder::new();
    scroll_widget.layout.pad(50.0, &root_widget.layout);
    scroll_widget.layout.dimensions(Dimensions { width: 300.0, height: 300.0 });
    scroll_widget.layout.scrollable = true;
    scroll_widget.event_handlers.push(Box::new(ScrollHandler::new()));

    let mut list_widget = WidgetBuilder::new();
    list_widget.layout.match_width(&scroll_widget.layout);
    list_widget.event_handlers.push(Box::new(WidgetScrollHandler::new()));

    let list_item_widgets = {
        let mut linear_layout = LinearLayout::new(Orientation::Vertical, &list_widget.layout);
        let mut list_item_widgets = Vec::new();
        for i in 1..15 {
            let mut list_item_widget = WidgetBuilder::new();
            let text_drawable = TextDrawable::new("hello".to_owned(), font_id);
            let text_dims = text_drawable.measure_dims_no_wrap(&resources);
            list_item_widget.set_drawable(widget::text::draw_text, Box::new(text_drawable));
            list_item_widget.layout.match_width(&list_widget.layout);
            list_item_widget.layout.height(text_dims.height);
            linear_layout.add_widget(&mut list_item_widget.layout);
            list_item_widgets.push(list_item_widget);
        }
        list_item_widgets
    };

    for list_item_widget in list_item_widgets {
        list_widget.add_child(Box::new(list_item_widget));
    }
    scroll_widget.add_child(Box::new(list_widget));
    root_widget.add_child(Box::new(scroll_widget));

    let ui = &mut Ui::new();
    ui.set_root(root_widget);

    let window_dims = ui.get_root_dims();
    window.window.window.set_inner_size(window_dims.width as u32, window_dims.height as u32);

    let mut glyph_cache = GlyphCache::new(&mut window.context.factory, 512, 512);
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
                    ui.draw(&resources, &mut glyph_cache, context, graphics);
                });
            }
        }
    }
}
