use graphics;
use graphics::types::Color;

use backend::gfx::G2d;
use backend::glyph::GlyphCache;
use graphics::Context;

use widget::drawable::Drawable;
use widget::property::PropSet;
use widget::style::{Styleable, Value};
use util::{Scalar, Rectangle};
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
    fn draw(&mut self, bounds: Rectangle, _: Rectangle, _: &mut GlyphCache, context: Context, graphics: &mut G2d) {
        let border = self.border.and_then(|(radius, color)| {
            Some(graphics::ellipse::Border {
                radius: radius,
                color: color,
            })
        });
        graphics::Ellipse::new(self.background_color)
            .maybe_border(border)
            .draw(bounds, &context.draw_state, context.transform, graphics);
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
                drawable.background_color = val.from_props(props)
            },
            EllipseStyleable::Border(ref val) => drawable.border = val.from_props(props),
        }
    }
}