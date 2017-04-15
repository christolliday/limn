#[macro_use]
extern crate limn;
extern crate backend;
extern crate graphics;

extern crate chrono;

mod util;

use std::{thread, time, f64};

use chrono::{Local, Timelike};
use graphics::types::Color;
use graphics::Context;

use backend::glyph::GlyphCache;
use backend::gfx::G2d;

use limn::widget::{WidgetBuilder, WidgetBuilderCore};
use limn::widget::drawable::{Drawable, DrawableEventHandler};
use limn::drawable::ellipse::{EllipseDrawable, EllipseStyleable};
use limn::event::{Target, Queue};
use limn::util::{Point, Rectangle, Dimensions, Scalar};
use limn::color::*;

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

        let style = style!(
            EllipseStyleable::BackgroundColor: WHITE,
            EllipseStyleable::Border: Some((2.0, BLACK)));
        let mut widget = WidgetBuilder::new();
        widget.set_drawable_with_style(EllipseDrawable::new(), style);
        widget.layout().dimensions(Dimensions {
            width: 200.0,
            height: 200.0,
        });

        let hour_angle = || 2.0 * f64::consts::PI * (Local::now().hour() % 12) as f64 / 12.0;
        let minute_angle = || 2.0 * f64::consts::PI * Local::now().minute() as f64 / 60.0;
        let second_angle = || 2.0 * f64::consts::PI * Local::now().second() as f64 / 60.0;
        let mut hour_widget = WidgetBuilder::new();
        hour_widget
            .set_drawable(HandDrawable::new(BLACK, 4.0, 60.0, hour_angle()))
            .add_handler(DrawableEventHandler::new(ClockTick, move |state: &mut HandDrawable| {
                state.angle = hour_angle()
            }));
        let mut minute_widget = WidgetBuilder::new();
        minute_widget
            .set_drawable(HandDrawable::new(BLACK, 3.0, 90.0, minute_angle()))
            .add_handler(DrawableEventHandler::new(ClockTick, move |state: &mut HandDrawable| {
                state.angle = minute_angle()
            }));
        let mut second_widget = WidgetBuilder::new();
        second_widget
            .set_drawable(HandDrawable::new(RED, 2.0, 80.0, second_angle()))
            .add_handler(DrawableEventHandler::new(ClockTick, move |state: &mut HandDrawable| {
                state.angle = second_angle()
            }));

        widget.add_child(hour_widget);
        widget.add_child(minute_widget);
        widget.add_child(second_widget);

        let clock_id = widget.id();
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
    clock.layout().center(&root_widget.layout());
    clock.layout().bound_by(&root_widget.layout()).padding(50.0);
    root_widget.add_child(clock);

    util::set_root_and_loop(window, ui, root_widget);
}
