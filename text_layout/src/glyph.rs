/// Logic and types specific to individual glyph layout.

use super::Font;
use types::{Range, Rectangle, Scalar};
use std;
use rusttype;
use rusttype::LayoutIter;
use super::line::LineInfo;

/// An iterator yielding the `Rect` for each `char`'s `Glyph` in the given `text`.
pub struct GlyphRects<'a, 'b> {
    /// The *y* axis `Range` of the `Line` for which character `Rect`s are being yielded.
    ///
    /// Every yielded `Rect` will use this as its `y` `Range`.
    y: Range,
    /// The position of the next `Rect`'s left edge along the *x* axis.
    next_left: Scalar,
    /// `PositionedGlyphs` yielded by the RustType `LayoutIter`.
    layout: LayoutIter<'a, 'b>,
}


impl<'a, 'b> Iterator for GlyphRects<'a, 'b> {
    type Item = Rectangle;
    fn next(&mut self) -> Option<Self::Item> {
        let GlyphRects { ref mut next_left, ref mut layout, y } = *self;
        layout.next().map(|g| {
            let left = *next_left;
            let right = g.pixel_bounding_box()
                .map(|bb| bb.max.x as Scalar)
                .unwrap_or_else(|| left + g.unpositioned().h_metrics().advance_width as Scalar);
            *next_left = right;
            let x = Range::new(left, right);
            Rectangle::from_ranges(x, y)
        })
    }
}

/// An iterator that, for every `(line, line_rect)` pair yielded by the given iterator,
/// produces an iterator that yields a `Rect` for every character in that line.
pub struct GlyphRectsPerLine<'a, I> {
    lines_with_rects: I,
    font: &'a Font,
    font_size: Scalar,
}

impl<'a, I> GlyphRectsPerLine<'a, I>
    where I: Iterator<Item = (&'a str, Rectangle)>
{
    /// Produce an iterator that, for every `(line, line_rect)` pair yielded by the given iterator,
    /// produces an iterator that yields a `Rect` for every character in that line.
    ///
    /// This is useful when information about character positioning is needed when reasoning about
    /// text layout.
    pub fn new(lines_with_rects: I, font: &'a Font, font_size: Scalar) -> GlyphRectsPerLine<'a, I> {
        GlyphRectsPerLine {
            lines_with_rects: lines_with_rects,
            font: font,
            font_size: font_size,
        }
    }
}
impl<'a, I> Iterator for GlyphRectsPerLine<'a, I>
    where I: Iterator<Item = (&'a str, Rectangle)>
{
    type Item = GlyphRects<'a, 'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let GlyphRectsPerLine { ref mut lines_with_rects, font, font_size } = *self;
        let scale = super::pt_to_scale(font_size);
        lines_with_rects.next().map(|(line_text, line_rect)| {
            let (x, y) = (line_rect.left as f32, line_rect.top as f32);
            let point = rusttype::Point { x: x, y: y };
            GlyphRects {
                next_left: line_rect.left,
                layout: font.layout(line_text, scale, point),
                y: line_rect.y_range(),
            }
        })
    }
}

/// Yields a `Rect` for each selected character in a single line of text.
///
/// This iterator can only be produced by the `SelectedCharRectsPerLine` iterator.
pub struct SelectedGlyphRects<'a, 'b> {
    enumerated_rects: std::iter::Enumerate<GlyphRects<'a, 'b>>,
    end_char_idx: usize,
}
impl<'a, 'b> Iterator for SelectedGlyphRects<'a, 'b> {
    type Item = Rectangle;
    fn next(&mut self) -> Option<Self::Item> {
        let SelectedGlyphRects { ref mut enumerated_rects, end_char_idx } = *self;
        enumerated_rects.next()
            .and_then(|(i, rect)| if i < end_char_idx { Some(rect) } else { None })
    }
}


/// Yields an iteraor yielding `Rect`s for each selected character in each line of text within
/// the given iterator yielding char `Rect`s.
///
/// Given some `start` and `end` indices, only `Rect`s for `char`s between these two indices
/// will be produced.
///
/// All lines that have no selected `Rect`s will be skipped.
pub struct SelectedGlyphRectsPerLine<'a, I> {
    enumerated_rects_per_line: std::iter::Enumerate<GlyphRectsPerLine<'a, I>>,
    start_cursor_idx: super::cursor::Index,
    end_cursor_idx: super::cursor::Index,
}

impl<'a, I> SelectedGlyphRectsPerLine<'a, I>
    where I: Iterator<Item = (&'a str, Rectangle)>
{
    /// Produces an iterator that yields iteraors yielding `Rect`s for each selected character in
    /// each line of text within the given iterator yielding char `Rect`s.
    ///
    /// Given some `start` and `end` indices, only `Rect`s for `char`s between these two indices
    /// will be produced.
    ///
    /// All lines that have no selected `Rect`s will be skipped.
    pub fn new(lines_with_rects: I,
               font: &'a Font,
               font_size: Scalar,
               start: super::cursor::Index,
               end: super::cursor::Index)
               -> SelectedGlyphRectsPerLine<'a, I> {
        SelectedGlyphRectsPerLine {
            enumerated_rects_per_line: GlyphRectsPerLine::new(lines_with_rects, font, font_size)
                .enumerate(),
            start_cursor_idx: start,
            end_cursor_idx: end,
        }
    }
}
impl<'a, I> Iterator for SelectedGlyphRectsPerLine<'a, I>
    where I: Iterator<Item = (&'a str, Rectangle)>
{
    type Item = SelectedGlyphRects<'a, 'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let SelectedGlyphRectsPerLine { ref mut enumerated_rects_per_line,
                                        start_cursor_idx,
                                        end_cursor_idx } = *self;

        enumerated_rects_per_line.next().map(|(i, rects)| {
            let end_char_idx =
                    // If this is the last line, the end is the char after the final selected char.
                    if i == end_cursor_idx.line {
                        end_cursor_idx.char
                    // Otherwise if in range, every char in the line is selected.
                    } else if start_cursor_idx.line <= i && i < end_cursor_idx.line {
                        std::u32::MAX as usize
                    // Otherwise if out of range, no chars are selected.
                    } else {
                        0
                    };

            let mut enumerated_rects = rects.enumerate();

            // If this is the first line, skip all non-selected chars.
            if i == start_cursor_idx.line {
                for _ in 0..start_cursor_idx.char {
                    enumerated_rects.next();
                }
            }

            SelectedGlyphRects {
                enumerated_rects: enumerated_rects,
                end_char_idx: end_char_idx,
            }
        })
    }
}


/// Find the index of the character that directly follows the cursor at the given `cursor_idx`.
///
/// Returns `None` if either the given `cursor::Index` `line` or `idx` fields are out of bounds
/// of the line information yielded by the `line_infos` iterator.
pub fn index_after_cursor<I>(mut line_infos: I, cursor_idx: super::cursor::Index) -> Option<usize>
    where I: Iterator<Item = LineInfo>
{
    line_infos.nth(cursor_idx.line)
        .and_then(|line_info| {
            let start_char = line_info.start_char;
            let end_char = line_info.end_char();
            let char_index = start_char + cursor_idx.char;
            if char_index <= end_char {
                Some(char_index)
            } else {
                None
            }
        })
}
