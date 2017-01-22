#[macro_use]
extern crate limn;
extern crate backend;
extern crate cassowary;
extern crate graphics;
extern crate input;
extern crate window;
extern crate find_folder;

extern crate chrono;

#[macro_use]
extern crate matches;

use limn::ui::*;
use limn::util::*;
use limn::widget::text::*;
use limn::widget;
use limn::event;
use limn::resources::Id;

use limn::event::{Event, Signal};

use limn::widget::DrawArgs;
use limn::widget::builder::WidgetBuilder;
use limn::widget::primitives::{RectDrawable, EllipseDrawable};
use limn::widget::button::ButtonBuilder;
use limn::widget::layout::{LinearLayout, Orientation};
use limn::widget::DrawableEventHandler;
use limn::widget::{EventHandler, EventArgs};

use limn::eventbus::{EventBus, EventAddress};

use backend::glyph::GlyphCache;
use backend::{Window, WindowEvents};
use input::ResizeEvent;
use backend::events::WindowEvent;

use graphics::types::Color;
use graphics::color::*;
use limn::color::*;

use std::thread;
use std::time;
use std::any::Any;
use std::f64;

use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, channel, Sender, Receiver};

use chrono::*;

use input::EventId;
const WIDGET_EVENT: usize = 0;

fn hour_angle() -> f64 {
    2.0 * f64::consts::PI * (Local::now().hour() % 12) as f64 / 12.0
}
fn minute_angle() -> f64 {
    2.0 * f64::consts::PI * Local::now().minute() as f64 / 60.0
}
fn second_angle() -> f64 {
    2.0 * f64::consts::PI * Local::now().second() as f64 / 60.0
}
const CLOCK_TICK: EventId = EventId("CLOCK_TICK");
struct ClockBuilder {
    widget: WidgetBuilder,
}
impl ClockBuilder {
    fn new(resources: &mut Resources, mut event_queue: EventQueue) -> Self {

        let circle = EllipseDrawable { background: WHITE, border: Some(graphics::ellipse::Border { color: BLACK, radius: 2.0 }) };
        let mut widget = WidgetBuilder::new();
        widget.set_drawable(widget::primitives::draw_ellipse, Box::new(circle));
        widget.layout.dimensions(Dimensions { width: 200.0, height: 200.0 });

        struct HandDrawable {
            background: Color,
            width: Scalar,
            length: Scalar,
            angle: Scalar, // radians
        }
        pub fn draw_clock_hand(draw_args: DrawArgs) {
            let DrawArgs { state, bounds, context, graphics, .. } = draw_args;
            let state: &HandDrawable = state.downcast_ref().unwrap();

            let cos = state.angle.cos();
            let sin = state.angle.sin();
            let hand_dir = Point { x: sin * 1.0, y: -cos * 1.0} * state.length;
            let hand_norm = Point { x: -cos * 1.0, y: -sin * 1.0} * state.width;
            let center = bounds.center();
            let points: Vec<[f64; 2]> = [
                center + hand_norm,
                center + hand_norm + hand_dir,
                center - hand_norm + hand_dir,
                center - hand_norm,
            ].iter().map(|point| { [point.x, point.y]}).collect();

            graphics::Polygon::new(state.background)
                .draw(&points, &context.draw_state, context.transform, graphics);
        }
        let hour_drawable = HandDrawable { background: BLACK, width: 4.0, length: 60.0, angle: hour_angle() };
        let mut hour_widget = WidgetBuilder::new();
        hour_widget.set_drawable(draw_clock_hand, Box::new(hour_drawable));
        let minute_drawable = HandDrawable { background: BLACK, width: 3.0, length: 90.0, angle: minute_angle() };
        let mut minute_widget = WidgetBuilder::new();
        minute_widget.set_drawable(draw_clock_hand, Box::new(minute_drawable));
        let second_drawable = HandDrawable { background: RED, width: 2.0, length: 80.0, angle: second_angle() };
        let mut second_widget = WidgetBuilder::new();
        second_widget.set_drawable(draw_clock_hand, Box::new(second_drawable));

        let clock_id = resources.widget_id();
        widget.set_id(clock_id);

        fn update_hour_hand(state: &mut HandDrawable) {
            state.angle = hour_angle();
        };
        fn update_minute_hand(state: &mut HandDrawable) {
            state.angle = minute_angle();
        };
        fn update_second_hand(state: &mut HandDrawable) {
            state.angle = second_angle();
        };

        struct ClockEventHandler {}
        impl EventHandler for ClockEventHandler {
            fn event_id(&self) -> EventId {
                CLOCK_TICK
            }
            fn handle_event(&mut self, event_args: EventArgs) {
                let EventArgs { widget_id, event_queue, .. } = event_args;
                event_queue.push(EventAddress::IdAddress("CHILD".to_owned(), widget_id.0), Box::new(Signal::new(CLOCK_TICK)));
            }
        }
        widget.event_handlers.push(Box::new(ClockEventHandler {}));

        hour_widget.event_handlers.push(Box::new(DrawableEventHandler::new(CLOCK_TICK, update_hour_hand)));
        minute_widget.event_handlers.push(Box::new(DrawableEventHandler::new(CLOCK_TICK, update_minute_hand)));
        second_widget.event_handlers.push(Box::new(DrawableEventHandler::new(CLOCK_TICK, update_second_hand)));

        widget.add_child(Box::new(hour_widget));
        widget.add_child(Box::new(minute_widget));
        widget.add_child(Box::new(second_widget));

        thread::spawn(move || {
            loop {
                thread::sleep(time::Duration::from_millis(1000));
                event_queue.push(EventAddress::IdAddress("CHILD".to_owned(), clock_id.0), Box::new(Signal::new(CLOCK_TICK)));
            }
        });

        ClockBuilder { widget: widget }
    }
    pub fn builder(self) -> WidgetBuilder {
        self.widget
    }
}

fn main() {
    let window_dims = Dimensions { width: 100.0, height: 100.0 };
    let mut window = Window::new("Limn clock demo", window_dims, Some(window_dims));
    let ui = &mut Ui::new();
    ui.event_queue.set_window(&window);
    let mut resources = Resources::new();

    let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
    let font_path = assets.join("fonts/Hack/Hack-Regular.ttf");

    let font_id = resources.fonts.insert_from_file(font_path).unwrap();

    let mut root_widget = WidgetBuilder::new();
    
    let mut clock = ClockBuilder::new(&mut resources, ui.event_queue.clone());
    clock.widget.layout.center(&root_widget.layout);
    clock.widget.layout.pad(50.0, &root_widget.layout);

    root_widget.add_child(Box::new(clock.builder()));
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
