use std::f64;

use euclid::{self, Point2D, Size2D, Vector2D};

use rusttype;
use graphics::{self, Context};

pub use graphics::types::{Color, Scalar};

use backend::gfx::G2d;

use text_layout;

pub type Size = Size2D<f64>;
pub type Point = Point2D<f64>;
pub type Vector = Vector2D<f64>;
pub type Rect = euclid::Rect<f64>;

pub trait RectExt<T> {
    fn from_text_layout(rect: text_layout::Rectangle) -> Self;
    fn from_rusttype<S: Into<T>>(rect: rusttype::Rect<S>) -> Self;
    fn to_text_layout(&self) -> text_layout::Rectangle;
    fn to_slice(&self) -> [T; 4];
    fn left(&self) -> T;
    fn top(&self) -> T;
    fn right(&self) -> T;
    fn bottom(&self) -> T;
    fn width(&self) -> T;
    fn height(&self) -> T;
    fn center(&self) -> Point;
    fn shrink_bounds(&self, size: T) -> Self;
}
impl RectExt<f64> for Rect {
    fn from_text_layout(rect: text_layout::Rectangle) -> Self {
        Rect::new(Point::new(rect.left, rect.top), Size::new(rect.width, rect.height))
    }
    fn from_rusttype<S: Into<f64>>(rect: rusttype::Rect<S>) -> Self {
        let origin = Point::new(rect.min.x.into(), rect.min.y.into());
        let size = Size::new(rect.max.x.into() - origin.x, rect.max.y.into() - origin.y);
        Rect::new(origin, size)
    }
    fn to_text_layout(&self) -> text_layout::Rectangle {
        text_layout::Rectangle {
            left: self.left(),
            top: self.top(),
            width: self.width(),
            height: self.height(),
        }
    }
    fn to_slice(&self) -> [f64; 4] {
        [self.left(), self.top(), self.width(), self.height()]
    }
    fn left(&self) -> f64 {
        self.origin.x
    }
    fn top(&self) -> f64 {
        self.origin.y
    }
    fn right(&self) -> f64 {
        self.origin.x + self.size.width
    }
    fn bottom(&self) -> f64 {
        self.origin.y + self.size.height
    }
    fn width(&self) -> f64 {
        self.size.width
    }
    fn height(&self) -> f64 {
        self.size.height
    }
    fn center(&self) -> Point {
        Point::new(self.left() + self.width() / 2.0, self.top() + self.height() / 2.0)
    }
    fn shrink_bounds(&self, size: f64) -> Self {
        Rect::new(
            Point::new(self.origin.x + size / 2.0, self.origin.y + size / 2.0),
            Size::new(self.size.width - size, self.size.height - size))
    }
}

pub trait SizeExt<T> {
    fn from_array(size: [u32; 2]) -> Self;
    fn from_tuple(size: (u32, u32)) -> Self;
    fn from_text_layout(rect: text_layout::Dimensions) -> Self;
}
impl SizeExt<f64> for Size {
    fn from_array(size: [u32; 2]) -> Self {
        Size::new(size[0] as f64, size[1] as f64)
    }
    fn from_tuple(size: (u32, u32)) -> Self {
        Size::new(size.0 as f64, size.1 as f64)
    }
    fn from_text_layout(rect: text_layout::Dimensions) -> Self {
        Size::new(rect.width, rect.height)
    }
}

pub fn mouse_inside_ellipse(mouse: Point, bounds: Rect) -> bool {
    let radius = Size::new(bounds.width() / 2.0, bounds.height() / 2.0);
    let center = Point::new(bounds.left() + radius.width, bounds.top() + radius.height);
    point_inside_ellipse(mouse, center, radius)
}
pub fn point_inside_ellipse(point: Point, center: Point, radius: Size) -> bool {
    (point.x - center.x).powi(2) / radius.width.powi(2) +
    (point.y - center.y).powi(2) / radius.height.powi(2) <= 1.0
}

// Retrieve the "dots per inch" factor by dividing the window width by the view.
#[allow(dead_code)]
fn get_dpi(context: &Context) -> f32 {
    let view_size = context.get_view_size();
    context.viewport
        .map(|v| v.window_size[0] as f32 / view_size[0] as f32)
        .unwrap_or(1.0)
}

pub fn draw_rect_outline(rect: Rect, color: Color, context: Context, graphics: &mut G2d) {
    let points = [[rect.left(), rect.top()],
                  [rect.right(), rect.top()],
                  [rect.right(), rect.bottom()],
                  [rect.left(), rect.bottom()],
                  [rect.left(), rect.top()]];
    let mut points = points.iter();
    if let Some(first) = points.next() {
        let line = graphics::Line::new_round(color, 1.0);
        let mut start = first;
        for end in points {
            let coords = [start[0], start[1], end[0], end[1]];
            line.draw(coords, &context.draw_state, context.transform, graphics);
            start = end;
        }
    }
}

pub fn crop_context(context: Context, rect: Rect) -> Context {
    let scissor_bounds = [rect.left() as u32, rect.top() as u32, rect.width() as u32, rect.height() as u32];
    Context { draw_state: context.draw_state.scissor(scissor_bounds), ..context }
}
