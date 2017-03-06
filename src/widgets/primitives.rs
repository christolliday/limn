use std::f64::consts::PI;

use graphics;
use graphics::types::Color;

use backend::gfx::G2d;
use backend::glyph::GlyphCache;
use graphics::Context;

use widget::drawable::{Drawable, StyleArgs, DrawArgs};
use widget::property::PropSet;
use widget::style::{self, Value, StyleField};
use util::{Scalar, Rectangle, Point};
use color::*;


pub struct RectDrawable {
    pub background_color: Color,
    pub corner_radius: Option<Scalar>,
}
impl Default for RectDrawable {
    fn default() -> Self {
        RectDrawable {
            background_color: WHITE,
            corner_radius: None,
        }
    }
}
impl RectDrawable {
    pub fn new() -> Self {
        RectDrawable::default()
    }
}
impl Drawable for RectDrawable {
    fn draw(&mut self, bounds: Rectangle, crop_to: Rectangle, glyph_cache: &mut GlyphCache, context: Context, graphics: &mut G2d) {
        if let Some(radius) = self.corner_radius {
            let points_per_corner = 8;
            let angle_per_step = 2.0 * PI / (points_per_corner * 4) as Scalar;
            fn circle_coords(radius: f64, step: f64, angle_per_step: f64) -> [f64; 2] {
                [radius * (step * angle_per_step).cos(), radius * (step * angle_per_step).sin()]
            };
            // corners are center points of four circle segments
            let inner_rect = Rectangle {
                left: bounds.left + radius,
                top: bounds.top + radius,
                width: bounds.width - 2.0 * radius,
                height: bounds.height - 2.0 * radius,
            };
            let points: Vec<[f64; 2]> = (0..4)
                .flat_map(|corner| {
                    let center: Point = match corner {
                        0 => inner_rect.bottom_right(),
                        1 => inner_rect.bottom_left(),
                        2 => inner_rect.top_left(),
                        3 => inner_rect.top_right(),
                        _ => unreachable!(),
                    };
                    let step_offset: u32 = corner * points_per_corner;
                    (0..points_per_corner + 1).map(move |corner_step| {
                        let circle_step = step_offset + corner_step;
                        let circle_offset = circle_coords(radius, circle_step as f64, angle_per_step);
                        [center.x + circle_offset[0], center.y + circle_offset[1]]
                    })
                })
                .collect();
            graphics::Polygon::new(self.background_color)
                .draw(&points, &context.draw_state, context.transform, graphics);
        } else {
            graphics::Rectangle::new(self.background_color)
                .draw(bounds, &context.draw_state, context.transform, graphics);
        }
    }
}

#[derive(Clone)]
pub enum RectStyleField {
    BackgroundColor(Value<Color>),
    CornerRadius(Value<Option<Scalar>>),
}

impl StyleField<RectDrawable> for RectStyleField {
    fn apply(&self, drawable: &mut RectDrawable, props: &PropSet) {
        match *self {
            RectStyleField::BackgroundColor(ref val) => {
                drawable.background_color = val.from_props(props)
            }
            RectStyleField::CornerRadius(ref val) => drawable.corner_radius = val.from_props(props),
        }
    }
}

pub struct EllipseDrawable {
    pub background_color: Color,
    pub border: Option<graphics::ellipse::Border>,
}
impl EllipseDrawable {
    pub fn new(background_color: Color, border: Option<graphics::ellipse::Border>) -> Self {
        EllipseDrawable {
            background_color: background_color,
            border: border,
        }
    }
}
impl Drawable for EllipseDrawable {
    fn draw(&mut self, bounds: Rectangle, crop_to: Rectangle, glyph_cache: &mut GlyphCache, context: Context, graphics: &mut G2d) {
        graphics::Ellipse::new(self.background_color)
            .maybe_border(self.border)
            .draw(bounds, &context.draw_state, context.transform, graphics);
    }
}
