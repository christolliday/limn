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
use limn::resources::Id;

use limn::widget::builder::WidgetBuilder;
use limn::widget::primitives::{RectDrawable};
use limn::widget::button::ToggleButtonBuilder;
use limn::widget::layout::{LinearLayout, Orientation};
use limn::widget::DrawableEventHandler;

use backend::glyph::GlyphCache;
use backend::{Window, WindowEvents};
use input::ResizeEvent;
use backend::events::WindowEvent;

use graphics::types::Color;

fn main() {
    let mut resources = Resources::new();

    let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
    let font_path = assets.join("fonts/Hack/Hack-Regular.ttf");

    let font_id = resources.fonts.insert_from_file(font_path).unwrap();
    
    let mut root_widget = WidgetBuilder::new();
    
    {
        let mut button = ToggleButtonBuilder::new(&mut resources);
        button.set_text("ON", "OFF", font_id, 20.0, [0.0, 0.0, 0.0, 1.0]);
        button.widget.layout.center(&root_widget.layout);
        button.widget.layout.pad(50.0, &root_widget.layout);

        root_widget.add_child(Box::new(button.builder()));
    }

    let ui = &mut Ui::new();
    ui.set_root(root_widget, &mut resources);

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
