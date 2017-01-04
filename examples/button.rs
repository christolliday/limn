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

use limn::widget::builder::WidgetBuilder;
use limn::widget::primitives::{RectDrawable};
use limn::widget::button::ToggleEventHandler;
use limn::widget::layout::{LinearLayout, Orientation};
use limn::widget::DrawableEventHandler;

use backend::glyph::GlyphCache;
use backend::{Window, WindowEvents};
use input::ResizeEvent;
use backend::events::WindowEvent;

fn main() {
    let mut resources = Resources::new();

    let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
    let font_path = assets.join("fonts/Hack/Hack-Regular.ttf");

    let font_id = resources.fonts.insert_from_file(font_path).unwrap();
    
    let mut root_widget = WidgetBuilder::new();
    
    let rect = RectDrawable { background: [1.0, 0.0, 0.0, 1.0] };
    let mut button_widget = WidgetBuilder::new();
    button_widget.set_drawable(widget::primitives::draw_rect, Box::new(rect));
    button_widget.event_handlers.push(Box::new(ToggleEventHandler::new()));

    fn set_rect_on(state: &mut RectDrawable) {
        state.background = [4.0, 1.0, 1.0, 1.0];
    };
    fn set_rect_off(state: &mut RectDrawable) {
        state.background = [1.0, 0.0, 0.0, 1.0];
    };
    button_widget.event_handlers.push(Box::new(DrawableEventHandler::new(event::BUTTON_ENABLED, set_rect_on)));
    button_widget.event_handlers.push(Box::new(DrawableEventHandler::new(event::BUTTON_DISABLED, set_rect_off)));
    button_widget.debug_color([1.0, 1.0, 0.0, 1.0]);
    button_widget.layout.dimensions(Dimensions { width: 100.0, height: 50.0 });
    button_widget.layout.center(&root_widget.layout);
    button_widget.layout.pad(50.0, &root_widget.layout);

    fn set_text_on(state: &mut TextDrawable) {
        state.text = "ON".to_owned();
    };
    fn set_text_off(state: &mut TextDrawable) {
        state.text = "OFF".to_owned();
    };
    let button_text_drawable = TextDrawable { text: "OFF".to_owned(), font_id: font_id, font_size: 20.0, text_color: [0.0,0.0,0.0,1.0], background_color: [0.0, 0.0, 0.0, 0.0] };
    let button_text_dims = button_text_drawable.measure_dims_no_wrap(&resources);
    let mut button_text_widget = WidgetBuilder::new();
    button_text_widget.set_drawable(widget::text::draw_text, Box::new(button_text_drawable));
    button_text_widget.event_handlers.push(Box::new(DrawableEventHandler::new(event::BUTTON_ENABLED, set_text_on)));
    button_text_widget.event_handlers.push(Box::new(DrawableEventHandler::new(event::BUTTON_DISABLED, set_text_off)));
    button_text_widget.layout.dimensions(button_text_dims);
    button_text_widget.layout.center(&button_widget.layout);

    button_widget.add_child(Box::new(button_text_widget));
    root_widget.add_child(Box::new(button_widget));

    let ui = &mut Ui::new();
    ui.set_root(root_widget);

    let window_dims = ui.get_root_dims();
    // Construct the window.
    let mut window = Window::new("Limn button demo", window_dims, Some(window_dims));
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
