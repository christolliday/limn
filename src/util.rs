use std::f64;

use euclid::{self, Point2D, Size2D};

use rusttype;
use graphics::{self, Context};
use graphics::types;

pub use graphics::types::{Color, Scalar};

use backend::gfx::G2d;

pub type Size = Size2D<f64>;
pub type Point = Point2D<f64>;
pub type Rect = euclid::Rect<f64>;

pub trait RectBounds<T> {
    fn left(&self) -> T;
    fn top(&self) -> T;
    fn right(&self) -> T;
    fn bottom(&self) -> T;
    fn width(&self) -> T;
    fn height(&self) -> T;
    fn center(&self) -> Point;
    fn shrink_bounds(&self, size: f64) -> Self;
    fn to_slice(&self) -> [f64; 4];
}
impl RectBounds<f64> for Rect {
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
    fn to_slice(&self) -> [f64; 4] {
        [self.left(), self.top(), self.width(), self.height()]
    }
}

pub fn point_inside_rect(point: Point, rect: Rect) -> bool {
    point.x > rect.left() && point.y > rect.top() && point.x < rect.left() + rect.width() &&
    point.y < rect.top() + rect.height()
}

pub fn mouse_inside_ellipse(mouse: Point, bounds: Rect) -> bool {
    let radius = Size::new(
        bounds.width() / 2.0,
        bounds.height() / 2.0,
    );
    let center = Point::new(
        bounds.left() + radius.width,
        bounds.top() + radius.height
    );
    point_inside_ellipse(mouse, center, radius)
}
pub fn point_inside_ellipse(point: Point, center: Point, radius: Size) -> bool {
    (point.x - center.x).powi(2) / radius.width.powi(2) +
    (point.y - center.y).powi(2) / radius.height.powi(2) <= 1.0
}

pub fn mul_size(val: [f64; 4], rhs: Size) -> [f64; 4] {
    [val[0] * rhs.width, val[1] * rhs.height, val[2] * rhs.width, val[3] * rhs.height]
}

pub fn map_rect_i32(rect: rusttype::Rect<i32>) -> types::Rectangle {
    [rect.min.x as f64,
     rect.min.y as f64,
     (rect.max.x - rect.min.x) as f64,
     (rect.max.y - rect.min.y) as f64]
}
pub fn map_rect_f32(rect: rusttype::Rect<f32>) -> types::Rectangle {
    [rect.min.x as f64,
     rect.min.y as f64,
     (rect.max.x - rect.min.x) as f64,
     (rect.max.y - rect.min.y) as f64]
}

// for text_layout interop, kludgey
use text_layout;
pub fn rect_from_text_layout(rect: text_layout::Rectangle) -> Rect {
    Rect::new(Point::new(rect.left, rect.top), Size::new(rect.width, rect.height))
}
pub fn text_layout_rect(rect: Rect) -> text_layout::Rectangle {
    text_layout::Rectangle {
        left: rect.left(),
        top: rect.top(),
        width: rect.width(),
        height: rect.height(),
    }
}
pub fn size_from_text_layout(rect: text_layout::Dimensions) -> Size {
    Size::new(rect.width, rect.height)
}


pub fn size_from_slice(size: [u32; 2]) -> Size {
    Size::new(
        size[0] as f64,
        size[1] as f64,
    )
}

pub fn size_from_tuple(size: (u32, u32)) -> Size {
    Size::new(
        size.0 as f64,
        size.1 as f64,
    )
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

// get smallest shared region
pub fn crop_rect(outer: Rect, inner: Rect) -> Rect {
    let top = f64::max(outer.top(), inner.top());
    let left = f64::max(outer.left(), inner.left());
    let right = f64::min(outer.right(), inner.right());
    let bottom = f64::min(outer.bottom(), inner.bottom());
    let width = f64::max(0.0, right - left);
    let height = f64::max(0.0, bottom - top);
    Rect::new(Point::new(left, top), Size::new(width, height))
}
