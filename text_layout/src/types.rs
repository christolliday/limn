use euclid;

// in logical pixels
pub type Size = euclid::Size2D<f32>;
pub type Point = euclid::Point2D<f32>;
pub type Vector = euclid::Vector2D<f32>;
pub type Rect = euclid::Rect<f32>;

pub trait RectExt<T> {
    fn left(&self) -> T;
    fn top(&self) -> T;
    fn right(&self) -> T;
    fn bottom(&self) -> T;
    fn width(&self) -> T;
    fn height(&self) -> T;
    fn from_ranges(x: Range, y: Range) -> Self;
    fn x_range(&self) -> Range;
    fn y_range(&self) -> Range;
}
impl RectExt<f32> for Rect {
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
    fn from_ranges(x: Range, y: Range) -> Self {
        Rect::new(Point::new(x.start, y.start), Size::new(x.end - x.start, y.end - y.start))
    }
    fn x_range(&self) -> Range {
        Range::new(self.left(), self.right())
    }
    fn y_range(&self) -> Range {
        Range::new(self.top(), self.bottom())
    }
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

impl Default for Align {
    fn default() -> Self {
        Align::Start
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Range {
    /// The start of some `Range` along an axis.
    pub start: f32,
    /// The end of some `Range` along an axis.
    pub end: f32,
}

impl Range {
    pub fn new(start: f32, end: f32) -> Range {
        Range {
            start: start,
            end: end,
        }
    }
    pub fn from_pos_and_len(pos: f32, len: f32) -> Range {
        let half_len = len / 2.0;
        let start = pos - half_len;
        let end = pos + half_len;
        Range::new(start, end)
    }
    pub fn middle(&self) -> f32 {
        (self.end + self.start) / 2.0
    }
    pub fn is_over(&self, pos: f32) -> bool {
        let Range { start, end } = self.undirected();
        pos >= start && pos <= end
    }
    pub fn has_same_direction(self, other: Self) -> bool {
        let self_direction = self.start <= self.end;
        let other_direction = other.start <= other.end;
        self_direction == other_direction
    }
    pub fn shift(self, amount: f32) -> Range {
        Range {
            start: self.start + amount,
            end: self.end + amount,
        }
    }
    pub fn undirected(self) -> Range {
        if self.start > self.end {
            self.invert()
        } else {
            self
        }
    }
    pub fn invert(self) -> Range {
        Range {
            start: self.end,
            end: self.start,
        }
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
