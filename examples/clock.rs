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

use limn::event::Event;

use limn::widget::DrawArgs;
use limn::widget::builder::WidgetBuilder;
use limn::widget::primitives::{RectDrawable, EllipseDrawable};
use limn::widget::button::ButtonBuilder;
use limn::widget::layout::{LinearLayout, Orientation};
use limn::widget::DrawableEventHandler;

use limn::eventbus::{EventBus, EventAddress};

use backend::glyph::GlyphCache;
use backend::{Window, WindowEvents};
use input::ResizeEvent;
use backend::events::WindowEvent;

use graphics::types::Color;

use std::thread;
use std::time;
use std::any::Any;
use std::f64;

use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, channel, Sender, Receiver};

use chrono::*;

use input::EventId;
const WIDGET_EVENT: usize = 0;

const CLOCK_TICK: EventId = EventId("CLOCK_TICK");
struct ClockBuilder {
    widget: WidgetBuilder,
    hour_id: Id,
    minute_id: Id,
    second_id: Id,
}
impl ClockBuilder {
    fn new(resources: &mut Resources, event_bus: &EventBus, sender: Sender<i32>) -> Self {

        let circle = EllipseDrawable { background: [1.0, 0.0, 1.0, 1.0] };
        let mut widget = WidgetBuilder::new();
        widget.set_drawable(widget::primitives::draw_ellipse, Box::new(circle));
        widget.layout.dimensions(Dimensions { width: 200.0, height: 200.0 });

        struct HandDrawable {
            background: Color,
            angle: Scalar, // radians
        }
        pub fn draw_clock_hand(draw_args: DrawArgs) {
            let DrawArgs { state, bounds, context, graphics, .. } = draw_args;
            let state: &HandDrawable = state.downcast_ref().unwrap();

            let cos = state.angle.cos();
            let sin = state.angle.sin();
            let point = Point { x: sin * 1.0, y: -cos * 1.0};
            let par = Point { x: -cos * 1.0, y: -sin * 1.0};
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
        let minute_drawable = HandDrawable { background: [1.0, 1.0, 0.0, 1.0], angle: 0.0 };
        let mut minute_widget = WidgetBuilder::new();
        minute_widget.set_drawable(draw_clock_hand, Box::new(minute_drawable));
        let second_drawable = HandDrawable { background: [1.0, 1.0, 0.0, 1.0], angle: 0.0 };
        let mut second_widget = WidgetBuilder::new();
        second_widget.set_drawable(draw_clock_hand, Box::new(second_drawable));

        let hour_id = resources.widget_id();
        hour_widget.set_id(hour_id);
        let minute_id = resources.widget_id();
        minute_widget.set_id(minute_id);
        let second_id = resources.widget_id();
        second_widget.set_id(second_id);

        fn update_hour_hand(state: &mut HandDrawable) {
            let local: DateTime<Local> = Local::now();
            let hour = (local.hour() % 12);
            state.angle = 2.0 * f64::consts::PI * hour as f64 / 12.0;
        };
        fn update_minute_hand(state: &mut HandDrawable) {
            let local: DateTime<Local> = Local::now();
            let minute = local.minute();
            state.angle = 2.0 * f64::consts::PI * minute as f64 / 60.0;
        };
        fn update_second_hand(state: &mut HandDrawable) {
            let local: DateTime<Local> = Local::now();
            let second = local.second();
            state.angle = 2.0 * f64::consts::PI * second as f64 / 60.0;
        };

        hour_widget.event_handlers.push(Box::new(DrawableEventHandler::new(CLOCK_TICK, update_hour_hand)));
        minute_widget.event_handlers.push(Box::new(DrawableEventHandler::new(CLOCK_TICK, update_minute_hand)));
        second_widget.event_handlers.push(Box::new(DrawableEventHandler::new(CLOCK_TICK, update_second_hand)));

        widget.add_child(Box::new(hour_widget));
        widget.add_child(Box::new(minute_widget));
        widget.add_child(Box::new(second_widget));

        thread::spawn(move || {
            loop {
                thread::sleep(time::Duration::from_millis(1000));
                sender.send(0);
            }
        });

        ClockBuilder { widget: widget, hour_id: hour_id, minute_id: minute_id, second_id: second_id }
    }
    pub fn builder(self) -> WidgetBuilder {
        self.widget
    }
}

event!(ClockEvent, i32);

fn main() {
    let mut resources = Resources::new();

    let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
    let font_path = assets.join("fonts/Hack/Hack-Regular.ttf");

    let font_id = resources.fonts.insert_from_file(font_path).unwrap();

    let mut event_bus = EventBus::new();

    let (tx, rx): (Sender<i32>, Receiver<i32>) = mpsc::channel();

    let mut root_widget = WidgetBuilder::new();
    
    let mut clock = ClockBuilder::new(&mut resources, &event_bus, tx);
    clock.widget.layout.center(&root_widget.layout);
    clock.widget.layout.pad(50.0, &root_widget.layout);

    let hour_id = clock.hour_id;
    let minute_id = clock.minute_id;
    let second_id = clock.second_id;

    root_widget.add_child(Box::new(clock.builder()));

    let ui = &mut Ui::new();
    ui.set_root(root_widget);

    event_bus.register_address(EventAddress::Id(0), |droog: (Id, usize)| {
        println!("{:?}", droog);
    });


    let window_dims = ui.get_root_dims();
    // Construct the window.
    let mut window = Window::new("Limn clock demo", window_dims, Some(window_dims));
    let mut glyph_cache = GlyphCache::new(&mut window.context.factory, 512, 512);


    let window_proxy = window.window.window.create_window_proxy();
    let event_queue = Arc::new(Mutex::new(Vec::new()));
    {
        let event_queue = event_queue.clone();
        thread::spawn(move || {
            loop {
                rx.recv();
                let mut queue = event_queue.lock().unwrap();
                queue.push(1);
                window_proxy.wakeup_event_loop();
            }
        });
    }

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
                let mut queue = event_queue.lock().unwrap();
                while queue.len() > 0 {
                    queue.pop();
                    let event = ClockEvent::new(CLOCK_TICK, 20);
                    ui.send_event(hour_id, event);
                    let event = ClockEvent::new(CLOCK_TICK, 20);
                    ui.send_event(minute_id, event);
                    let event = ClockEvent::new(CLOCK_TICK, 20);
                    ui.send_event(second_id, event);
                }
                window.draw_2d(|context, graphics| {
                    graphics::clear([0.8, 0.8, 0.8, 1.0], graphics);
                    ui.draw(&resources, &mut glyph_cache, context, graphics);
                });
            }
        }
    }
}
