extern crate limn;
extern crate backend;
extern crate graphics;

extern crate chrono;

mod util;

use std::thread;
use std::time;
use std::f64;

use chrono::*;
use graphics::types::Color;
use graphics::Context;

use backend::glyph::GlyphCache;
use backend::gfx::G2d;

use limn::widget::drawable::{Drawable, DrawableEventHandler};
use limn::widget::builder::WidgetBuilder;
use limn::drawable::ellipse::EllipseDrawable;
use limn::event::{Target, Queue};
use limn::color::*;
use limn::util::{Point, Rectangle, Dimensions, Scalar};

fn hour_angle() -> f64 {
    2.0 * f64::consts::PI * (Local::now().hour() % 12) as f64 / 12.0
}
fn minute_angle() -> f64 {
    2.0 * f64::consts::PI * Local::now().minute() as f64 / 60.0
}
fn second_angle() -> f64 {
    2.0 * f64::consts::PI * Local::now().second() as f64 / 60.0
}
struct ClockTick;

pub struct HandDrawable {
    color: Color,
    width: Scalar,
    length: Scalar,
    angle: Scalar, // radians
}
impl HandDrawable {
    pub fn new(color: Color, width: Scalar, length: Scalar, angle: Scalar) -> Self {
        HandDrawable {
            color: color,
            width: width,
            length: length,
            angle: angle,
        }
    }
}
impl Drawable for HandDrawable {
    fn draw(&mut self, bounds: Rectangle, _: Rectangle, _: &mut GlyphCache, context: Context, graphics: &mut G2d) {
        let cos = self.angle.cos();
        let sin = self.angle.sin();
        let hand_dir = Point {
            x: sin * 1.0,
            y: -cos * 1.0,
        } * self.length;
        let hand_norm = Point {
            x: -cos * 1.0,
            y: -sin * 1.0,
        } * self.width;
        let center = bounds.center();
        let points: Vec<[f64; 2]> = [center + hand_norm,
                                     center + hand_norm + hand_dir,
                                     center - hand_norm + hand_dir,
                                     center - hand_norm]
            .iter()
            .map(|point| [point.x, point.y])
            .collect();
        graphics::Polygon::new(self.color)
            .draw(&points, &context.draw_state, context.transform, graphics);
    }
}

struct ClockBuilder {
    widget: WidgetBuilder,
}
impl ClockBuilder {
    fn new(mut queue: Queue) -> Self {

        let border = graphics::ellipse::Border {
            color: BLACK,
            radius: 2.0,
        };
        let drawable = EllipseDrawable::new(WHITE, Some(border));
        let mut widget = WidgetBuilder::new().set_drawable(drawable);
        widget.layout.dimensions(Dimensions {
            width: 200.0,
            height: 200.0,
        });

        

        fn update_hour_hand(state: &mut HandDrawable) {
            state.angle = hour_angle();
        };
        fn update_minute_hand(state: &mut HandDrawable) {
            state.angle = minute_angle();
        };
        fn update_second_hand(state: &mut HandDrawable) {
            state.angle = second_angle();
        };

        let hour_widget = WidgetBuilder::new()
            .set_drawable(HandDrawable::new(BLACK, 4.0, 60.0, hour_angle()))
            .add_handler(DrawableEventHandler::new(ClockTick, update_hour_hand));
        let minute_widget = WidgetBuilder::new()
            .set_drawable(HandDrawable::new(BLACK, 3.0, 90.0, minute_angle()))
            .add_handler(DrawableEventHandler::new(ClockTick, update_minute_hand));
        let second_widget = WidgetBuilder::new()
            .set_drawable(HandDrawable::new(RED, 2.0, 80.0, second_angle()))
            .add_handler(DrawableEventHandler::new(ClockTick, update_second_hand));

        widget.add_child(hour_widget);
        widget.add_child(minute_widget);
        widget.add_child(second_widget);

        let clock_id = widget.id;
        thread::spawn(move || loop {
            thread::sleep(time::Duration::from_millis(1000));
            queue.push(Target::SubTree(clock_id), ClockTick);
        });

        ClockBuilder { widget: widget }
    }
}

fn main() {
    let (window, ui) = util::init_default("Limn clock demo");

    let mut root_widget = WidgetBuilder::new();
    let mut clock = ClockBuilder::new(ui.queue.clone()).widget;
    clock.layout.center(&root_widget);
    clock.layout.bound_by(&root_widget, Some(50.0));
    root_widget.add_child(clock);

    util::set_root_and_loop(window, ui, root_widget);
}
