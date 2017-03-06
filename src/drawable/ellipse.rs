use std::f64::consts::PI;

use graphics;
use graphics::types::Color;

use backend::gfx::G2d;
use backend::glyph::GlyphCache;
use graphics::Context;

use widget::drawable::Drawable;
use widget::property::PropSet;
use widget::style::{self, Value, StyleField};
use util::{Scalar, Rectangle, Point};
use color::*;

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
