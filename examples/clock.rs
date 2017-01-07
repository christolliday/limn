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

use limn::widget::DrawArgs;
use limn::widget::builder::WidgetBuilder;
use limn::widget::primitives::{RectDrawable, EllipseDrawable};
use limn::widget::button::ButtonBuilder;
use limn::widget::layout::{LinearLayout, Orientation};
use limn::widget::DrawableEventHandler;

use backend::glyph::GlyphCache;
use backend::{Window, WindowEvents};
use input::ResizeEvent;
use backend::events::WindowEvent;

use graphics::types::Color;

use std::thread;
use std::time::Duration;

struct ClockBuilder {
    widget: WidgetBuilder,
}
impl ClockBuilder {
    fn new() -> Self {

        let circle = EllipseDrawable { background: [1.0, 0.0, 1.0, 1.0] };
        let mut widget = WidgetBuilder::new();
        widget.set_drawable(widget::primitives::draw_ellipse, Box::new(circle));
        widget.layout.dimensions(Dimensions { width: 200.0, height: 200.0 });

        struct HandDrawable {
            background: Color,
            angle: Scalar,
        }
        pub fn draw_clock_hand(draw_args: DrawArgs) {
            let DrawArgs { state, bounds, context, graphics, .. } = draw_args;
            let state: &HandDrawable = state.downcast_ref().unwrap();

            let cos = state.angle.cos();
            let sin = state.angle.sin();
            let point = Point { x: - sin * 1.0, y: cos * 1.0};
            let par = Point { x: cos * 1.0, y: sin * 1.0};
            let width = 10.0;
            let length = 100.0;
            let center = bounds.center();
            let points: Vec<[f64; 2]> = [
                center + (par * width),
                center + (par * width) + (point * length),
                center - (par * width) + (point * length),
                center - (par * width),
            ].iter().map(|point| { [point.x, point.y]}).collect();

            graphics::Polygon::new(state.background)
                .draw(&points, &context.draw_state, context.transform, graphics);
        }
        let hour_drawable = HandDrawable { background: [1.0, 1.0, 0.0, 1.0], angle: 0.0 };
        let mut hour_widget = WidgetBuilder::new();
        hour_widget.set_drawable(draw_clock_hand, Box::new(hour_drawable));
        let minute_drawable = HandDrawable { background: [1.0, 0.0, 1.0, 1.0], angle: 0.0 };
        let mut minute_widget = WidgetBuilder::new();
        minute_widget.set_drawable(draw_clock_hand, Box::new(minute_drawable));

        widget.add_child(Box::new(hour_widget));
        widget.add_child(Box::new(minute_widget));

        thread::spawn(|| {
            loop {
                thread::sleep(Duration::from_millis(1000));
                //let state: &HandDrawable = widget.drawable.unwrap().downcast_ref().unwrap();
            }
        });

        ClockBuilder { widget: widget }
    }
    pub fn builder(self) -> WidgetBuilder {
        self.widget
    }
}

fn main() {
    let mut resources = Resources::new();

    let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
    let font_path = assets.join("fonts/Hack/Hack-Regular.ttf");

    let font_id = resources.fonts.insert_from_file(font_path).unwrap();
    
    let mut root_widget = WidgetBuilder::new();
    
    let mut clock = ClockBuilder::new();
    clock.widget.layout.center(&root_widget.layout);
    clock.widget.layout.pad(50.0, &root_widget.layout);

    root_widget.add_child(Box::new(clock.builder()));

    let ui = &mut Ui::new();
    ui.set_root(root_widget);

    let window_dims = ui.get_root_dims();
    // Construct the window.
    let mut window = Window::new("Limn clock demo", window_dims, Some(window_dims));
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
