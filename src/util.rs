use std::ops::{Mul, Div, Add, Sub};

use rusttype;
use graphics::{self, Context};
use graphics::types;

pub use graphics::types::{Color, Scalar};

use backend::gfx::G2d;

#[derive(Copy, Clone, Debug)]
pub struct Dimensions {
    pub width: Scalar,
    pub height: Scalar,
}
#[derive(Copy, Clone, Debug)]
pub struct Point {
    pub x: Scalar,
    pub y: Scalar,
}
#[derive(Copy, Clone, Debug)]
pub struct Rectangle {
    pub top: Scalar,
    pub left: Scalar,
    pub width: Scalar,
    pub height: Scalar,
}

impl Into<(u32, u32)> for Dimensions {
    fn into(self) -> (u32, u32) {
        (self.width as u32, self.height as u32)
    }
}
impl Into<[f64; 2]> for Dimensions {
    fn into(self) -> [f64; 2] {
        [self.width, self.height]
    }
}
impl Into<Dimensions> for [u32; 2] {
    fn into(self) -> Dimensions {
        Dimensions {
            width: self[0] as f64,
            height: self[1] as f64,
        }
    }
}
impl Into<Dimensions> for (u32, u32) {
    fn into(self) -> Dimensions {
        Dimensions {
            width: self.0 as f64,
            height: self.1 as f64,
        }
    }
}
impl Into<Point> for [f64; 2] {
    fn into(self) -> Point {
        Point {
            x: self[0],
            y: self[1],
        }
    }
}
impl Into<types::Rectangle> for Rectangle {
    fn into(self) -> types::Rectangle {
        [self.left, self.top, self.width, self.height]
    }
}
impl Into<[u32; 4]> for Rectangle {
    fn into(self) -> [u32; 4] {
        [self.left as u32, self.top as u32, self.width as u32, self.height as u32]
    }
}

pub fn point_inside_rect(point: Point, rect: Rectangle) -> bool {
    point.x > rect.left && point.y > rect.top && point.x < rect.left + rect.width &&
    point.y < rect.top + rect.height
}

pub fn mouse_inside_ellipse(mouse: Point, bounds: Rectangle) -> bool {
    let radius = Dimensions {
        width: bounds.width / 2.0,
        height: bounds.height / 2.0,
    };
    let center = Point {
        x: bounds.left + radius.width,
        y: bounds.top + radius.height,
    };
    point_inside_ellipse(mouse, center, radius)
}
pub fn point_inside_ellipse(point: Point, center: Point, radius: Dimensions) -> bool {
    (point.x - center.x).powi(2) / radius.width.powi(2) +
    (point.y - center.y).powi(2) / radius.height.powi(2) <= 1.0
}

impl Rectangle {
    pub fn new(left: Scalar, top: Scalar, width: Scalar, height: Scalar) -> Self {
        Rectangle {
            left: left,
            top: top,
            width: width,
            height: height,
        }
    }
    pub fn new_empty() -> Self {
        Rectangle::new(0.0, 0.0, 0.0, 0.0)
    }
    pub fn right(&self) -> Scalar {
        self.left + self.width
    }
    pub fn bottom(&self) -> Scalar {
        self.top + self.height
    }
    pub fn top_left(&self) -> Point {
        Point { x: self.left, y: self.top }
    }
    pub fn top_right(&self) -> Point {
        Point { x: self.right(), y: self.top }
    }
    pub fn bottom_left(&self) -> Point {
        Point { x: self.left, y: self.bottom() }
    }
    pub fn bottom_right(&self) -> Point {
        Point { x: self.right(), y: self.bottom() }
    }
    pub fn dims(&self) -> Dimensions {
        Dimensions {
            width: self.width,
            height: self.height,
        }
    }
    pub fn center(&self) -> Point {
        Point {
            x: self.left + self.width / 2.0,
            y: self.top + self.height / 2.0,
        }
    }
    /// true if either width or height are exactly 0
    pub fn no_area(&self) -> bool {
        return self.width == 0.0 || self.height == 0.0;
    }
}
impl Div<Dimensions> for Dimensions {
    type Output = Self;
    fn div(self, rhs: Dimensions) -> Self {
        Dimensions {
            width: self.width / rhs.width,
            height: self.height / rhs.height,
        }
    }
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
impl Mul<Dimensions> for types::Rectangle {
    type Output = Self;
    fn mul(self, rhs: Dimensions) -> Self {
        [self[0] * rhs.width, self[1] * rhs.height, self[2] * rhs.width, self[3] * rhs.height]
    }
}
impl Add<Point> for Point {
    type Output = Self;
    fn add(self, rhs: Point) -> Self {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
impl Sub<Point> for Point {
    type Output = Self;
    fn sub(self, rhs: Point) -> Self {
        Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}
impl Mul<Scalar> for Point {
    type Output = Self;
    fn mul(self, rhs: Scalar) -> Self {
        Point {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

// for text_layout interop, kludgey
use text_layout;
impl Into<text_layout::Rectangle> for Rectangle {
    fn into(self) -> text_layout::Rectangle {
        text_layout::Rectangle {
            left: self.left,
            top: self.top,
            width: self.width,
            height: self.height,
        }
    }
}
impl Into<Dimensions> for text_layout::Dimensions {
    fn into(self) -> Dimensions {
        Dimensions {
            width: self.width,
            height: self.height,
        }
    }
}

// Retrieve the "dots per inch" factor by dividing the window width by the view.
#[allow(dead_code)]
fn get_dpi(context: &Context) -> f32 {
    let view_size = context.get_view_size();
    context.viewport
        .map(|v| v.window_size[0] as f32 / view_size[0] as f32)
        .unwrap_or(1.0)
}

pub fn draw_rect_outline(rect: Rectangle, color: Color, context: Context, graphics: &mut G2d) {
    let points = [[rect.left, rect.top],
                  [rect.right(), rect.top],
                  [rect.right(), rect.bottom()],
                  [rect.left, rect.bottom()],
                  [rect.left, rect.top]];
    let mut points = points.iter();
    if let Some(first) = points.next() {
        let line = graphics::Line::new_round(color, 2.0);
        let mut start = first;
        for end in points {
            let coords = [start[0], start[1], end[0], end[1]];
            line.draw(coords, &context.draw_state, context.transform, graphics);
            start = end;
        }
    }
}

pub fn crop_context(context: Context, rect: Rectangle) -> Context {
    let scissor_bounds = [rect.left as u32,
                          rect.top as u32,
                          rect.width as u32,
                          rect.height as u32];
    Context { draw_state: context.draw_state.scissor(scissor_bounds), ..context }
}

// get smallest shared region
pub fn crop_rect(outer: Rectangle, inner: Rectangle) -> Rectangle {
    let top = f64::max(outer.top, inner.top);
    let left = f64::max(outer.left, inner.left);
    let right = f64::min(outer.left + outer.width, inner.left + inner.width);
    let bottom = f64::min(outer.top + outer.height, inner.top + inner.height);
    Rectangle {
        top: top,
        left: left,
        width: f64::max(0.0, right - left),
        height: f64::max(0.0, bottom - top),
    }
}
