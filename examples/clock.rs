#[macro_use]
extern crate limn;
#[macro_use]
extern crate limn_layout;
extern crate webrender_api;
extern crate euclid;

extern crate chrono;

mod util;

use std::f32;
use std::{thread, time};

use chrono::{Local, Timelike};
use webrender_api::*;

use limn::prelude::*;
use limn::drawable::ellipse::{EllipseDrawable, EllipseStyleable};

type Radians = euclid::Radians<f32>;

struct ClockTick;

pub struct HandDrawable {
    color: Color,
    width: f32,
    length: f32,
    rotation: Radians,
}
impl HandDrawable {
    pub fn new(color: Color, width: f32, length: f32, rotation: Radians) -> Self {
        HandDrawable {
            color: color,
            width: width,
            length: length,
            rotation: rotation,
        }
    }
}

impl Drawable for HandDrawable {
    fn draw(&mut self, bounds: Rect, _: Rect, renderer: &mut RenderBuilder) {
        let transform = rotation_transform(&bounds.center().typed(),
            self.rotation + Radians::new(f32::consts::PI));
        renderer.builder.push_stacking_context(
            ScrollPolicy::Fixed,
            Rect::zero().typed(),
            Some(PropertyBinding::Value(transform)),
            TransformStyle::Flat,
            None,
            MixBlendMode::Normal,
            Vec::new(),
        );
        renderer.builder.push_rect(
            Rect::new(
                bounds.center() + Size::new(-self.width / 2.0, 0.0),
                Size::new(self.width, self.length)
            ).typed(),
            None, self.color.into());
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
    widget: Widget,
}
impl ClockBuilder {
    fn new() -> Self {

        let style = style!(
            EllipseStyleable::BackgroundColor: WHITE,
            EllipseStyleable::Border: Some((2.0, BLACK)));
        let mut widget = Widget::new();
        widget.set_drawable_with_style(EllipseDrawable::new(), style);
        layout!(widget: size(Size::new(200.0, 200.0)));

        let hour_angle = || rotation((Local::now().hour() % 12) as f32 / 12.0);
        let minute_angle = || rotation(Local::now().minute() as f32 / 60.0);
        let second_angle = || rotation(Local::now().second() as f32 / 60.0);
        let mut hour_widget = Widget::new();
        hour_widget
            .set_drawable(HandDrawable::new(BLACK, 4.0, 60.0, hour_angle()))
            .add_handler(DrawableEventHandler::new(ClockTick, move |state: &mut HandDrawable| {
                state.rotation = hour_angle()
            }));
        let mut minute_widget = Widget::new();
        minute_widget
            .set_drawable(HandDrawable::new(BLACK, 3.0, 90.0, minute_angle()))
            .add_handler(DrawableEventHandler::new(ClockTick, move |state: &mut HandDrawable| {
                state.rotation = minute_angle()
            }));
        let mut second_widget = Widget::new();
        second_widget
            .set_drawable(HandDrawable::new(RED, 2.0, 80.0, second_angle()))
            .add_handler(DrawableEventHandler::new(ClockTick, move |state: &mut HandDrawable| {
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
    let mut root = app.ui.root.clone();

    let mut clock = ClockBuilder::new().widget;
    layout!(clock:
        center(&root),
        bound_by(&root).padding(50.0));
    root.add_child(clock.clone());

    thread::spawn(move || loop {
        thread::sleep(time::Duration::from_millis(1000));
        event_global(ClockTick);
    });
    app.add_handler_fn(move |_: &ClockTick, _| {
        clock.event_subtree(ClockTick);
    });
    app.main_loop();
}
