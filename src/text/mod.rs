//! Text layout logic.

pub mod font;
pub mod cursor;
pub mod glyph;
pub mod line;

use std;
use util::*;
use rusttype;
use rusttype::Scale;
use self::font::Font;

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

pub fn get_positioned_glyphs(text: &str,
                             rect: Rectangle,
                             font: &Font,
                             font_size: FontSize,
                             line_spacing: f64,
                             line_wrap: Wrap,
                             x_align: Align,
                             y_align: Align)
                             -> Vec<PositionedGlyph> {

    let line_infos: Vec<line::Info> = line::infos(text, font, font_size, line_wrap, rect.width).collect();
    let line_infos = line_infos.iter().cloned();
    let line_texts = line_infos.clone().map(|info| &text[info.byte_range()]);
    let line_rects = line::rects(line_infos, font_size, rect, x_align, y_align, line_spacing);

    let mut positioned_glyphs = Vec::new();
    for (line_text, line_rect) in line_texts.zip(line_rects) {
        let point = rusttype::Point {
            x: line_rect.left as f32,
            y: line_rect.top as f32,
        };
        println!("{:?} {:?}", line_text, point);
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
pub fn height(num_lines: usize, font_size: FontSize, line_spacing: Scalar) -> Scalar {
    if num_lines > 0 {
        num_lines as Scalar * font_size as Scalar + (num_lines - 1) as Scalar * line_spacing
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
pub fn pt_to_px(font_size_in_points: FontSize) -> f32 {
    font_size_in_points as f32
    //(font_size_in_points * 4) as f32 / 3.0
}

/// Converts the given font size in "points" to a uniform `rusttype::Scale`.
pub fn pt_to_scale(font_size_in_points: FontSize) -> Scale {
    Scale::uniform(pt_to_px(font_size_in_points))
}

