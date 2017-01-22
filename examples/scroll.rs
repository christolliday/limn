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
use limn::eventbus::EventBus;
use limn::color::*;

use backend::{Window, WindowEvents, OpenGL};
use backend::events::WindowEvent;
use backend::glyph::GlyphCache;
use input::{ResizeEvent, MouseCursorEvent, PressEvent, ReleaseEvent, Event, Input, EventId};

use cassowary::WeightedRelation::*;
use cassowary::strength::*;

use std::any::Any;

fn main() {
    let window_dims = Dimensions { width: 100.0, height: 100.0 };

    let mut window = Window::new("Limn scroll demo", window_dims, None);
    let mut resources = Resources::new();

    let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
    let font_path = assets.join("fonts/Hack/Hack-Regular.ttf");
    let image_path = assets.join("images/rust.png");

    let font_id = resources.fonts.insert_from_file(font_path).unwrap();
    let image_id = resources.images.insert_from_file(&mut window.context.factory, image_path);
    
    let mut root_widget = WidgetBuilder::new();

    let mut scroll_widget = WidgetBuilder::new();
    scroll_widget.layout.dimensions(Dimensions { width: 200.0, height: 200.0 });
    scroll_widget.layout.pad(100.0, &root_widget.layout);
    scroll_widget.layout.scrollable = true;
    scroll_widget.event_handlers.push(Box::new(ScrollHandler::new()));

    let mut rect_container_widget = WidgetBuilder::new();
    rect_container_widget.event_handlers.push(Box::new(WidgetScrollHandler::new()));
    rect_container_widget.layout.dimensions(Dimensions { width: 400.0, height: 400.0});

    let mut rect_tl_widget = WidgetBuilder::new();
    rect_tl_widget.set_drawable(widget::primitives::draw_rect, Box::new(RectDrawable { background: RED }));
    rect_tl_widget.layout.dimensions(Dimensions { width: 200.0, height: 200.0});
    rect_tl_widget.layout.align_top(&rect_container_widget.layout);
    rect_tl_widget.layout.align_left(&rect_container_widget.layout);

    let mut rect_tr_widget = WidgetBuilder::new();
    rect_tr_widget.set_drawable(widget::primitives::draw_rect, Box::new(RectDrawable { background: GREEN }));
    rect_tr_widget.layout.dimensions(Dimensions { width: 200.0, height: 200.0});
    rect_tr_widget.layout.align_top(&rect_container_widget.layout);
    rect_tr_widget.layout.align_right(&rect_container_widget.layout);

    let mut rect_bl_widget = WidgetBuilder::new();
    rect_bl_widget.set_drawable(widget::primitives::draw_rect, Box::new(RectDrawable { background: BLUE }));
    rect_bl_widget.layout.dimensions(Dimensions { width: 200.0, height: 200.0});
    rect_bl_widget.layout.align_bottom(&rect_container_widget.layout);
    rect_bl_widget.layout.align_left(&rect_container_widget.layout);

    let mut rect_br_widget = WidgetBuilder::new();
    rect_br_widget.set_drawable(widget::primitives::draw_rect, Box::new(RectDrawable { background: FUSCHIA }));
    rect_br_widget.layout.dimensions(Dimensions { width: 200.0, height: 200.0});
    rect_br_widget.layout.align_bottom(&rect_container_widget.layout);
    rect_br_widget.layout.align_right(&rect_container_widget.layout);

    rect_container_widget.add_child(Box::new(rect_tl_widget));
    rect_container_widget.add_child(Box::new(rect_tr_widget));
    rect_container_widget.add_child(Box::new(rect_bl_widget));
    rect_container_widget.add_child(Box::new(rect_br_widget));
    scroll_widget.add_child(Box::new(rect_container_widget));
    root_widget.add_child(Box::new(scroll_widget));

    let ui = &mut Ui::new();
    ui.set_root(root_widget, &mut resources);
    ui.resize_window_to_fit(&window);

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
