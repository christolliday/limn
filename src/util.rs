
use window::Size;
use graphics::types;
use graphics::types::{Scalar};

#[derive(Copy, Clone, Debug)]
pub struct Dimensions { pub width: Scalar, pub height: Scalar, }
#[derive(Copy, Clone, Debug)]
pub struct Point { pub x: Scalar, pub y: Scalar, }

impl Into<Size> for Dimensions {
    fn into(self) -> Size {
        Size { width: self.width as u32, height: self.height as u32 }
    }
}
impl Into<Point> for [f64; 2] {
    fn into(self) -> Point {
        Point { x: self[0], y: self[1] }
    }
}

pub fn point_inside_rect(point: Point, rect: types::Rectangle) -> bool {
    point.x > rect[0] && point.y > rect[1] && point.x < rect[0] + rect[2] && point.y < rect[1] + rect[3]
}
pub fn point_inside_ellipse(point: Point, center: Point, radius: Dimensions) -> bool {
    (point.x - center.x).powi(2) / radius.width.powi(2) + (point.y - center.y).powi(2) / radius.height.powi(2) <= 1.0
}