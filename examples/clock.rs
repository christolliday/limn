#[macro_use]
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
use std::time::Duration;
use std::any::Any;

use std::sync::mpsc::{self, channel, Sender, Receiver};

use input::EventId;
const WIDGET_EVENT: usize = 0;

const CLOCK_TICK: EventId = EventId("CLOCK_TICK");
struct ClockBuilder {
    widget: WidgetBuilder,
    hour_id: Id,
}
impl ClockBuilder {
    fn new(resources: &mut Resources, event_bus: &EventBus, sender: Sender<i32>) -> Self {

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

        let hour_id = resources.widget_id();
        hour_widget.set_id(hour_id);

        fn update_hand(state: &mut HandDrawable) {
            state.angle = 30.0;
        };

        //event_bus.register_address(EventAddress::Id(hour_id.0), update_hand);
        hour_widget.event_handlers.push(Box::new(DrawableEventHandler::new(CLOCK_TICK, update_hand)));

        widget.add_child(Box::new(hour_widget));
        widget.add_child(Box::new(minute_widget));


        /*fn set_rect_off(angle: Scalar, state: &mut HandDrawable) {
            state.background = [1.0, 0.0, 0.0, 1.0];
        };
        hour_widget.event_handlers.push(Box::new(DrawableEventHandler::new(ANGLE_CHANGED, set_rect_on)));

        let event_bus.register();*/

        thread::spawn(move || {
            let sec = 0;
            loop {
                thread::sleep(Duration::from_millis(1000));
                //event_bus.post_id(WIDGET_EVENT, sec);
                sender.send(sec);
                //event_bus.post_id(WIDGET_EVENT, (hour_id, sec));
                //sec += 1;
                //let state: &HandDrawable = widget.drawable.unwrap().downcast_ref().unwrap();
            }
        });

        ClockBuilder { widget: widget, hour_id: hour_id }
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
                let res = rx.try_recv();
                if res.is_ok() {
                    let event = ClockEvent::new(CLOCK_TICK, 20);
                    ui.send_event(hour_id, event);
                }
                window.draw_2d(|context, graphics| {
                    graphics::clear([0.8, 0.8, 0.8, 1.0], graphics);
                    ui.draw(&resources, &mut glyph_cache, context, graphics);
                });
            }
        }
    }
}
