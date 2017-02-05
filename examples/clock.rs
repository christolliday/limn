#[macro_use]
extern crate limn;
extern crate graphics;

extern crate chrono;

mod util;

use std::thread;
use std::time;
use std::f64;

use chrono::*;
use graphics::types::Color;

use limn::widget::{Drawable, DrawArgs, DrawableEventHandler};
use limn::widget::builder::WidgetBuilder;
use limn::widgets::primitives;
use limn::event::{EventId, EventAddress, EventQueue};
use limn::color::*;
use limn::util::{Point, Dimensions, Scalar};

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
    fn new(mut event_queue: EventQueue) -> Self {

        let drawable = primitives::ellipse_drawable(WHITE,
                                                    Some(graphics::ellipse::Border {
                                                        color: BLACK,
                                                        radius: 2.0,
                                                    }));
        let mut widget = WidgetBuilder::new().set_drawable(drawable);
        widget.layout.dimensions(Dimensions {
            width: 200.0,
            height: 200.0,
        });

        struct HandDrawState {
            color: Color,
            width: Scalar,
            length: Scalar,
            angle: Scalar, // radians
        }
        pub fn draw_clock_hand(draw_args: DrawArgs) {
            let DrawArgs { state, bounds, context, graphics, .. } = draw_args;
            let state: &HandDrawState = state.downcast_ref().unwrap();

            let cos = state.angle.cos();
            let sin = state.angle.sin();
            let hand_dir = Point {
                x: sin * 1.0,
                y: -cos * 1.0,
            } * state.length;
            let hand_norm = Point {
                x: -cos * 1.0,
                y: -sin * 1.0,
            } * state.width;
            let center = bounds.center();
            let points: Vec<[f64; 2]> = [center + hand_norm,
                                         center + hand_norm + hand_dir,
                                         center - hand_norm + hand_dir,
                                         center - hand_norm]
                .iter()
                .map(|point| [point.x, point.y])
                .collect();

            graphics::Polygon::new(state.color)
                .draw(&points, &context.draw_state, context.transform, graphics);
        }
        fn hand_drawable(color: Color, width: Scalar, length: Scalar, angle: Scalar) -> Drawable {
            let draw_state = HandDrawState {
                color: color,
                width: width,
                length: length,
                angle: angle,
            };
            Drawable::new(Box::new(draw_state), draw_clock_hand)
        }

        fn update_hour_hand(state: &mut HandDrawState) {
            state.angle = hour_angle();
        };
        fn update_minute_hand(state: &mut HandDrawState) {
            state.angle = minute_angle();
        };
        fn update_second_hand(state: &mut HandDrawState) {
            state.angle = second_angle();
        };

        let hour_widget = WidgetBuilder::new()
            .set_drawable(hand_drawable(BLACK, 4.0, 60.0, hour_angle()))
            .add_handler(Box::new(DrawableEventHandler::new(CLOCK_TICK, update_hour_hand)));
        let minute_widget = WidgetBuilder::new()
            .set_drawable(hand_drawable(BLACK, 3.0, 90.0, minute_angle()))
            .add_handler(Box::new(DrawableEventHandler::new(CLOCK_TICK, update_minute_hand)));
        let second_widget = WidgetBuilder::new()
            .set_drawable(hand_drawable(RED, 2.0, 80.0, second_angle()))
            .add_handler(Box::new(DrawableEventHandler::new(CLOCK_TICK, update_second_hand)));

        widget.add_child(Box::new(hour_widget));
        widget.add_child(Box::new(minute_widget));
        widget.add_child(Box::new(second_widget));

        let clock_id = widget.id;
        thread::spawn(move || loop {
            thread::sleep(time::Duration::from_millis(1000));
            event_queue.push(EventAddress::SubTree(clock_id), CLOCK_TICK, Box::new(()));
        });

        ClockBuilder { widget: widget }
    }
}

fn main() {
    let (window, ui, event_queue) = util::init_default("Limn clock demo");

    let mut root_widget = WidgetBuilder::new();
    let mut clock = ClockBuilder::new(event_queue.clone()).widget;
    clock.layout.center(&root_widget.layout);
    clock.layout.bound_by(&root_widget.layout, Some(50.0));
    root_widget.add_child(Box::new(clock));

    util::set_root_and_loop(window, ui, root_widget, event_queue, vec!{});
}
