//! Text layout logic.

pub mod cursor;
pub mod glyph;
pub mod line;

use std;
use std::f64;
use util::*;
use rusttype;
use rusttype::Scale;
use super::resources::font::Font;
use self::line::{LineRects, LineInfo, LineInfos};

pub type FontSize = u32;
/// The RustType `PositionedGlyph` type used by conrod.
pub type PositionedGlyph = rusttype::PositionedGlyph<'static>;

/// The way in which text should wrap around the width.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Wrap {
    NoWrap,
    /// Wrap at the first character that exceeds the width.
    Character,
    /// Wrap at the first word that exceeds the width.
    Whitespace,
}

pub fn get_text_height(text: &str,
                       font: &Font,
                       font_size: Scalar,
                       line_height: Scalar,
                       max_width: Scalar,
                       line_wrap: Wrap,
                       x_align: Align,
                       y_align: Align)
                       -> Scalar {
    let line_infos: Vec<LineInfo> = LineInfos::new(text, font, font_size, line_wrap, max_width)
        .collect();
    line_infos.len() as f64 * line_height
}


pub fn get_text_dimensions(text: &str,
                           font: &Font,
                           font_size: Scalar,
                           line_height: Scalar,
                           x_align: Align,
                           y_align: Align)
                           -> Dimensions {

    let line_infos: Vec<LineInfo> = LineInfos::new(text, font, font_size, Wrap::NoWrap, f64::MAX)
        .collect();
    let line_infos = line_infos.iter().cloned();
    let line_texts = line_infos.clone().map(|info| &text[info.byte_range()]);

    let rect = Rectangle {
        top: 0.0,
        left: 0.0,
        width: f64::MAX,
        height: f64::MAX,
    };
    let line_rects = LineRects::new(line_infos, font_size, rect, x_align, y_align, line_height);

    let mut max_width = 0.0;
    for line_rect in line_rects.clone() {
        max_width = f64::max(max_width, line_rect.width);
    }
    Dimensions {
        width: max_width,
        height: line_rects.count() as f64 * line_height,
    }
}

pub fn get_positioned_glyphs(text: &str,
                             rect: Rectangle,
                             font: &Font,
                             font_size: Scalar,
                             line_height: Scalar,
                             line_wrap: Wrap,
                             x_align: Align,
                             y_align: Align)
                             -> Vec<PositionedGlyph> {

    let line_infos: Vec<LineInfo> = LineInfos::new(text, font, font_size, line_wrap, rect.width)
        .collect();
    let line_infos = line_infos.iter().cloned();
    let line_texts = line_infos.clone().map(|info| &text[info.byte_range()]);
    let line_rects = LineRects::new(line_infos, font_size, rect, x_align, y_align, line_height);

    let mut positioned_glyphs = Vec::new();
    for (line_text, line_rect) in line_texts.zip(line_rects) {
        let point = rusttype::Point {
            x: line_rect.left as f32,
            y: line_rect.top as f32 + font_size as f32,
        };
        positioned_glyphs.extend(font.layout(line_text, Scale::uniform(font_size as f32), point)
            .map(|g| g.standalone()));
    }
    positioned_glyphs
}

/// An iterator yielding each line within the given `text` as a new `&str`, where the start and end
/// indices into each line are provided by the given iterator.
#[derive(Clone)]
pub struct Lines<'a, I>
    where I: Iterator<Item = std::ops::Range<usize>>
{
    text: &'a str,
    ranges: I,
}


/// Determine the total height of a block of text with the given number of lines, font size and
/// `line_spacing` (the space that separates each line of text).
pub fn height(num_lines: usize, font_size: Scalar, line_height: Scalar) -> Scalar {
    if num_lines > 0 {
        num_lines as Scalar * line_height
        // num_lines as Scalar * font_size as Scalar + (num_lines - 1) as Scalar * line_spacing
    } else {
        0.0
    }
}


/// Produce an iterator yielding each line within the given `text` as a new `&str`, where the
/// start and end indices into each line are provided by the given iterator.
pub fn lines<I>(text: &str, ranges: I) -> Lines<I>
    where I: Iterator<Item = std::ops::Range<usize>>
{
    Lines {
        text: text,
        ranges: ranges,
    }
}

impl<'a, I> Iterator for Lines<'a, I>
    where I: Iterator<Item = std::ops::Range<usize>>
{
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        let Lines { text, ref mut ranges } = *self;
        ranges.next().map(|range| &text[range])
    }
}


/// Converts the given font size in "points" to its font size in pixels.
pub fn pt_to_px(font_size_in_points: Scalar) -> f32 {
    font_size_in_points as f32
}

/// Converts the given font size in "points" to a uniform `rusttype::Scale`.
pub fn pt_to_scale(font_size_in_points: Scalar) -> Scale {
    Scale::uniform(pt_to_px(font_size_in_points))
}
