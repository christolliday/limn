//! Geometric type definitions, such as `Point`, `Rect` and `Size`

use std::f32;

use euclid;
use rusttype;
use webrender::api::*;

/// This is the unit used for all geometric types that represent a position on screen,
/// independent of the device's hidpi factor, or "Device Pixel Ratio".
/// Sometimes known as 'points', logical pixels or device independent pixels,
///
/// `DensityIndependentPixel` is an alias to a webrender `LayerPixel` to simplify
/// integration, but doesn't imply that the reference frame is a layer. Positions
/// should generally be relative to the window, where the origin is the top left,
/// and x and y values increase moving down and to the right.
pub type DensityIndependentPixel = LayerPixel;

pub type Size = euclid::TypedSize2D<f32, DensityIndependentPixel>;
pub type Point = euclid::TypedPoint2D<f32, DensityIndependentPixel>;
pub type Vector = euclid::TypedVector2D<f32, DensityIndependentPixel>;
pub type Rect = euclid::TypedRect<f32, DensityIndependentPixel>;

/// This is the unit of actual pixels in the framebuffer.
/// Multiply by the windows hidpi factor to get `DensityIndependentPixel`s
pub use webrender::api::DevicePixel;

/// Extension trait for rectangles.
/// Helper methods for rectangle sides depend on the assumption that `Rect`
/// size never contains negative values.
pub trait RectExt<T> {
    fn from_rusttype<S: Into<T>>(rect: rusttype::Rect<S>) -> Self;
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
impl RectExt<f32> for Rect {
    fn from_rusttype<S: Into<f32>>(rect: rusttype::Rect<S>) -> Self {
        let origin = Point::new(rect.min.x.into(), rect.min.y.into());
        let size = Size::new(rect.max.x.into() - origin.x, rect.max.y.into() - origin.y);
        Rect::new(origin, size)
    }
    fn to_slice(&self) -> [f32; 4] {
        [self.left(), self.top(), self.width(), self.height()]
    }
    fn left(&self) -> f32 {
        self.origin.x
    }
    fn top(&self) -> f32 {
        self.origin.y
    }
    fn right(&self) -> f32 {
        self.origin.x + self.size.width
    }
    fn bottom(&self) -> f32 {
        self.origin.y + self.size.height
    }
    fn width(&self) -> f32 {
        self.size.width
    }
    fn height(&self) -> f32 {
        self.size.height
    }
    fn center(&self) -> Point {
        Point::new(self.left() + self.width() / 2.0, self.top() + self.height() / 2.0)
    }
    fn shrink_bounds(&self, size: f32) -> Self {
        Rect::new(
            Point::new(self.origin.x + size / 2.0, self.origin.y + size / 2.0),
            Size::new(self.size.width - size, self.size.height - size))
    }
}

/// Extension trait for sizes.
pub trait SizeExt<T> {
    fn from_array(size: [u32; 2]) -> Self;
    fn from_tuple(size: (u32, u32)) -> Self;
}

impl SizeExt<f32> for Size {
    fn from_array(size: [u32; 2]) -> Self {
        Size::new(size[0] as f32, size[1] as f32)
    }
    fn from_tuple(size: (u32, u32)) -> Self {
        Size::new(size.0 as f32, size.1 as f32)
    }
}
