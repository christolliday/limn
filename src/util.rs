use window::Size;
use graphics::types;

pub use graphics::types::Scalar;
use rusttype;
use graphics::Context;

/*#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Px(pub f64);
#[derive(Copy, Clone, Debug)]
pub struct Dp(pub f64);*/

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

/// The orientation of **Align**ment along some **Axis**.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Align {
    /// **Align** our **Start** with the **Start** of some other widget along the **Axis**.
    Start,
    /// **Align** our **Middle** with the **Middle** of some other widget along the **Axis**.
    Middle,
    /// **Align** our **End** with the **End** of some other widget along the **Axis**.
    End,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Range {
    /// The start of some `Range` along an axis.
    pub start: Scalar,
    /// The end of some `Range` along an axis.
    pub end: Scalar,
}
impl Range {
    pub fn new(start: Scalar, end: Scalar) -> Range {
        Range {
            start: start,
            end: end,
        }
    }
    pub fn from_pos_and_len(pos: Scalar, len: Scalar) -> Range {
        let half_len = len / 2.0;
        let start = pos - half_len;
        let end = pos + half_len;
        Range::new(start, end)
    }
    pub fn middle(&self) -> Scalar {
        (self.end + self.start) / 2.0
    }
    pub fn is_over(&self, pos: Scalar) -> bool {
        let Range { start, end } = self.undirected();
        pos >= start && pos <= end
    }
    pub fn has_same_direction(self, other: Self) -> bool {
        let self_direction = self.start <= self.end;
        let other_direction = other.start <= other.end;
        self_direction == other_direction
    }
    pub fn shift(self, amount: Scalar) -> Range {
        Range { start: self.start + amount, end: self.end + amount }
    }
    pub fn undirected(self) -> Range {
        if self.start > self.end { self.invert() } else { self }
    }
    pub fn invert(self) -> Range {
        Range { start: self.end, end: self.start }
    }
    pub fn align_start_of(self, other: Self) -> Self {
        let diff = if self.has_same_direction(other) {
            other.start - self.start
        } else {
            other.start - self.end
        };
        self.shift(diff)
    }
    pub fn align_middle_of(self, other: Self) -> Self {
        let diff = other.middle() - self.middle();
        self.shift(diff)
    }
    pub fn align_end_of(self, other: Self) -> Self {
        let diff = if self.has_same_direction(other) {
            other.end - self.end
        } else {
            other.end - self.start
        };
        self.shift(diff)
    }
}

impl Into<Size> for Dimensions {
    fn into(self) -> Size {
        Size {
            width: self.width as u32,
            height: self.height as u32,
        }
    }
}
impl Into<[f64; 2]> for Dimensions {
    fn into(self) -> [f64; 2] {
        [self.width, self.height]
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

pub fn point_inside_rect(point: Point, rect: Rectangle) -> bool {
    point.x > rect.left && point.y > rect.top && point.x < rect.left + rect.width &&
    point.y < rect.top + rect.height
}
pub fn point_inside_ellipse(point: Point, center: Point, radius: Dimensions) -> bool {
    (point.x - center.x).powi(2) / radius.width.powi(2) +
    (point.y - center.y).powi(2) / radius.height.powi(2) <= 1.0
}

impl Rectangle {
    pub fn from_ranges(x: Range, y: Range) -> Self {
        Rectangle {
            left: x.start,
            top: y.start,
            width: x.end - x.start,
            height: y.end - y.start,
        }
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
use std::ops::Mul;
impl Mul<Dimensions> for types::Rectangle {
    type Output = Self;
    fn mul(self, rhs: Dimensions) -> Self {
        [self[0] * rhs.width, self[1] * rhs.height, self[2] * rhs.width, self[3] * rhs.height]
    }
}

// Retrieve the "dots per inch" factor by dividing the window width by the view.
fn get_dpi(context: &Context) -> f32 {
    let view_size = context.get_view_size();
    context.viewport
        .map(|v| v.window_size[0] as f32 / view_size[0] as f32)
        .unwrap_or(1.0)
}