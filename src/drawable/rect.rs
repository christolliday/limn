use std::f64::consts::PI;

use graphics;
use graphics::types::Color;

use backend::gfx::G2d;
use backend::glyph::GlyphCache;
use graphics::Context;

use widget::drawable::Drawable;
use widget::property::PropSet;
use widget::style::{Value, Styleable};
use util::{Scalar, Rect, Point, RectExt};
use color::*;


pub struct RectDrawable {
    pub background_color: Color,
    pub corner_radius: Option<Scalar>,
    pub border: Option<(Scalar, Color)>,
}
impl Default for RectDrawable {
    fn default() -> Self {
        RectDrawable {
            background_color: WHITE,
            corner_radius: None,
            border: None,
        }
    }
}
impl RectDrawable {
    pub fn new() -> Self {
        RectDrawable::default()
    }
}
impl Drawable for RectDrawable {
    fn draw(&mut self, mut bounds: Rect, _: Rect, _: &mut GlyphCache, context: Context, graphics: &mut G2d) {

        // using piston graphics, drawing borders and rounded edges is currently the largest performance bottleneck
        // todo: make it faster! probably will require replacing piston graphics
        if let Some((radius, _)) = self.border {
            // piston graphics draws the border outside the rectangle bounds
            // so it can get cut off by the clip rect, this shrinks the rect
            // to accomodate the border.
            bounds = bounds.shrink_bounds(radius * 2.0);
        }
        if let Some(radius) = self.corner_radius {
            let points_per_corner = 8;
            let angle_per_step = 2.0 * PI / (points_per_corner * 4) as Scalar;
            fn circle_coords(radius: f64, step: f64, angle_per_step: f64) -> [f64; 2] {
                [radius * (step * angle_per_step).cos(), radius * (step * angle_per_step).sin()]
            };
            // corners are center points of four circle arcs
            let inner_rect = bounds.shrink_bounds(radius * 2.0);
            let points: Vec<[f64; 2]> = (0..4)
                .flat_map(|corner| {
                    let center: Point = match corner {
                        0 => inner_rect.bottom_right(),
                        1 => inner_rect.bottom_left(),
                        2 => inner_rect.origin,
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
            if let Some((radius, color)) = self.border {
                let line = graphics::Line::new_round(color, radius);
                let mut points = points.iter().peekable();
                let first = points.peek().unwrap().clone();
                while let Some(val) = points.next() {
                    let next = points.peek().unwrap_or(&first);
                    let coords = [val[0], val[1], next[0], next[1]];
                    line.draw(coords, &context.draw_state, context.transform, graphics);
                }
            }
        } else {
            let border = self.border.and_then(|(radius, color)| {
                Some(graphics::rectangle::Border {
                    radius: radius,
                    color: color,
                })
            });
            graphics::Rectangle::new(self.background_color)
                .maybe_border(border)
                .draw(bounds.to_slice(), &context.draw_state, context.transform, graphics);
        }
    }
}

#[derive(Clone)]
pub enum RectStyleable {
    BackgroundColor(Value<Color>),
    CornerRadius(Value<Option<Scalar>>),
    Border(Value<Option<(Scalar, Color)>>),
}

impl Styleable<RectDrawable> for RectStyleable {
    fn apply(&self, drawable: &mut RectDrawable, props: &PropSet) {
        match *self {
            RectStyleable::BackgroundColor(ref val) => {
                drawable.background_color = val.get(props)
            }
            RectStyleable::CornerRadius(ref val) => drawable.corner_radius = val.get(props),
            RectStyleable::Border(ref val) => drawable.border = val.get(props),
        }
    }
}
