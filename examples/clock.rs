#[macro_use]
extern crate limn;
extern crate euclid;

extern crate chrono;

mod util;

use std::f32;
use std::{thread, time};

use chrono::{Local, Timelike};

use limn::webrender::api::*;
use limn::prelude::*;
use limn::draw::ellipse::{EllipseState, EllipseStyle};

type Radians = euclid::Radians<f32>;

struct ClockTick;

pub struct ClockHand {
    color: Color,
    width: f32,
    length: f32,
    rotation: Radians,
}
impl ClockHand {
    pub fn new(color: Color, width: f32, length: f32, rotation: Radians) -> Self {
        ClockHand {
            color: color,
            width: width,
            length: length,
            rotation: rotation,
        }
    }
}

impl Draw for ClockHand {
    fn draw(&mut self, bounds: Rect, _: Rect, renderer: &mut RenderBuilder) {
        let transform = rotation_transform(&bounds.center(),
            self.rotation + Radians::new(f32::consts::PI));
        renderer.builder.push_stacking_context(
            &PrimitiveInfo::new(Rect::zero()),
            ScrollPolicy::Fixed,
            Some(PropertyBinding::Value(transform)),
            TransformStyle::Flat,
            None,
            MixBlendMode::Normal,
            Vec::new(),
        );
        let rect = Rect::new(
            bounds.center() + Size::new(-self.width / 2.0, 0.0),
            Size::new(self.width, self.length)
        );
        renderer.builder.push_rect(
            &PrimitiveInfo::new(rect),
            self.color.into());
        renderer.builder.pop_stacking_context();
    }
}

fn rotation_transform(origin: &LayoutPoint, rotation: Radians) -> LayoutTransform {
    let pre_transform = LayoutTransform::create_translation(origin.x, origin.y, 0.0);
    let post_transform = LayoutTransform::create_translation(-origin.x, -origin.y, -0.0);
    let transform = LayoutTransform::identity().pre_rotate(0.0, 0.0, 1.0, -rotation);
    pre_transform.pre_mul(&transform).pre_mul(&post_transform)
}

fn rotation(fraction: f32) -> Radians {
    Radians::new(2.0 * f32::consts::PI * fraction)
}

struct ClockBuilder {
    widget: WidgetBuilder,
}
impl ClockBuilder {
    fn new() -> Self {

        let style = style!(
            EllipseStyle::BackgroundColor: WHITE,
            EllipseStyle::Border: Some((2.0, BLACK)));
        let mut widget = WidgetBuilder::new("clock");
        widget.set_draw_state_with_style(EllipseState::new(), style);
        widget.layout().add(size(Size::new(200.0, 200.0)));

        let hour_angle = || rotation((Local::now().hour() % 12) as f32 / 12.0);
        let minute_angle = || rotation(Local::now().minute() as f32 / 60.0);
        let second_angle = || rotation(Local::now().second() as f32 / 60.0);
        let mut hour_widget = WidgetBuilder::new("hours");
        hour_widget
            .set_draw_state(ClockHand::new(BLACK, 4.0, 60.0, hour_angle()))
            .add_handler(DrawEventHandler::new(ClockTick, move |state: &mut ClockHand| {
                state.rotation = hour_angle()
            }));
        let mut minute_widget = WidgetBuilder::new("minutes");
        minute_widget
            .set_draw_state(ClockHand::new(BLACK, 3.0, 90.0, minute_angle()))
            .add_handler(DrawEventHandler::new(ClockTick, move |state: &mut ClockHand| {
                state.rotation = minute_angle()
            }));
        let mut second_widget = WidgetBuilder::new("seconds");
        second_widget
            .set_draw_state(ClockHand::new(RED, 2.0, 80.0, second_angle()))
            .add_handler(DrawEventHandler::new(ClockTick, move |state: &mut ClockHand| {
                state.rotation = second_angle()
            }));

        widget
            .add_child(hour_widget)
            .add_child(minute_widget)
            .add_child(second_widget);

        ClockBuilder { widget: widget }
    }
}

fn main() {
    let mut app = util::init_default("Limn clock demo");
    let mut root = WidgetBuilder::new("root");

    let mut clock = ClockBuilder::new().widget;
    clock.layout().add(constraints![
        center(&root),
        bound_by(&root).padding(50.0),
    ]);
    let clock_ref = clock.widget_ref();
    root.add_child(clock);

    thread::spawn(move || loop {
        thread::sleep(time::Duration::from_millis(1000));
        event_global(ClockTick);
    });
    app.add_handler(move |_: &ClockTick, _: EventArgs| {
        clock_ref.event_subtree(ClockTick);
    });
    app.main_loop(root);
}
