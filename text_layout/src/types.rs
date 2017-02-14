
/// The type used for scalars.
pub type Scalar = f64;

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