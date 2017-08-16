use graphics::types::Color;

use render::RenderBuilder;
use widget::drawable::Drawable;
use widget::property::PropSet;
use widget::style::{Styleable, Value};
use util::{Scalar, Rect, RectExt};
use color::*;

pub struct EllipseDrawable {
    pub background_color: Color,
    pub border: Option<(Scalar, Color)>,
}
impl Default for EllipseDrawable {
    fn default() -> Self {
        EllipseDrawable {
            background_color: WHITE,
            border: None,
        }
    }
}

impl EllipseDrawable {
    pub fn new() -> Self {
        EllipseDrawable::default()
    }
}

impl Drawable for EllipseDrawable {
    fn draw(&mut self, bounds: Rect, _: Rect, renderer: &mut RenderBuilder) {
        /*let (bounds, border) = if let Some((radius, color)) = self.border {
            (bounds.shrink_bounds(radius), Some(graphics::ellipse::Border {
                radius: radius,
                color: color,
            }))
        } else {
            (bounds, None)
        };
        graphics::Ellipse::new(self.background_color)
            .maybe_border(border)
            .draw(bounds.to_slice(), &context.draw_state, context.transform, graphics);*/
    }
}


#[derive(Clone)]
pub enum EllipseStyleable {
    BackgroundColor(Value<Color>),
    Border(Value<Option<(Scalar, Color)>>),
}

impl Styleable<EllipseDrawable> for EllipseStyleable {
    fn apply(&self, drawable: &mut EllipseDrawable, props: &PropSet) {
        match *self {
            EllipseStyleable::BackgroundColor(ref val) => {
                drawable.background_color = val.get(props)
            },
            EllipseStyleable::Border(ref val) => drawable.border = val.get(props),
        }
    }
}
