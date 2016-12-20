
use window::Size;
use graphics::types;

pub use conrod::{Range, Align};
pub use graphics::types::{Scalar};
#[derive(Copy, Clone, Debug)]
pub struct Dimensions { pub width: Scalar, pub height: Scalar, }
#[derive(Copy, Clone, Debug)]
pub struct Point { pub x: Scalar, pub y: Scalar, }
#[derive(Copy, Clone, Debug)]
pub struct Rectangle { pub top: Scalar, pub left: Scalar, pub width: Scalar, pub height: Scalar, }

impl Into<Size> for Dimensions {
    fn into(self) -> Size {
        Size { width: self.width as u32, height: self.height as u32 }
    }
}
impl Into<[f64; 2]> for Dimensions {
    fn into(self) -> [f64; 2] {
        [ self.width, self.height ]
    }
}
impl Into<Point> for [f64; 2] {
    fn into(self) -> Point {
        Point { x: self[0], y: self[1] }
    }
}
impl Into<types::Rectangle> for Rectangle {
    fn into(self) -> types::Rectangle {
        [ self.left, self.top, self.width, self.height ]
    }
}

pub fn point_inside_rect(point: Point, rect: Rectangle) -> bool {
    point.x > rect.left && point.y > rect.top && point.x < rect.left + rect.width && point.y < rect.top + rect.height
}
pub fn point_inside_ellipse(point: Point, center: Point, radius: Dimensions) -> bool {
    (point.x - center.x).powi(2) / radius.width.powi(2) + (point.y - center.y).powi(2) / radius.height.powi(2) <= 1.0
}

impl Rectangle {
    pub fn from_ranges(x: Range, y: Range) -> Self {
        Rectangle { left: x.start, top: y.start, width: x.end - x.start, height: y.end - y.start }
    }
    pub fn x_range(&self) -> Range {
        Range::new(self.left, self.right())
    }
    pub fn y_range(&self) -> Range {
        Range::new(self.top, self.bottom())
    }
    pub fn right(&self) -> Scalar {
        self.left + self.width
    }
    pub fn bottom(&self) -> Scalar {
        self.top + self.height
    }
}