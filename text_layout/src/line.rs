
/// Text handling logic related to individual lines of text.
///
/// This module is the core of multi-line text handling.
use rusttype;
use super::Font;
use rusttype::Scale;
use types::{Rectangle, Scalar, Range, Align};
use std;
use rusttype::GlyphId;
use std::str::CharIndices;
use std::iter::Peekable;
use super::Wrap;
use super::glyph::SelectedGlyphRectsPerLine;

#[derive(Copy, Clone, Debug, PartialEq)]
enum BreakType {
    /// A break caused by the text exceeding some maximum width.
    Wrap {
        /// The byte length which should be skipped in order to reach the first non-whitespace
        /// character to use as the beginning of the next line.
        len_bytes: usize,
    },
    /// A break caused by a newline character.
    Newline {
        /// The width of the "newline" token in bytes.
        len_bytes: usize,
    },
    End,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Break {
    /// The byte index at which the line ends.
    byte: usize,
    /// The char index at which the line ends.
    char: usize,
    break_type: BreakType,
}
impl Break {
    fn new(byte: usize, char: usize, break_type: BreakType) -> Self {
        Break {
            byte: byte,
            char: char,
            break_type: break_type,
        }
    }
}

/// Information about a single line of text within a `&str`.
///
/// `Info` is a minimal amount of information that can be stored for efficient reasoning about
/// blocks of text given some `&str`. The `start` and `end_break` can be used for indexing into
/// the `&str`, and the `width` can be used for calculating line `Rect`s, alignment, etc.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct LineInfo {
    /// The index into the `&str` that represents the first character within the line.
    pub start_byte: usize,
    /// The character index of the first character in the line.
    pub start_char: usize,
    /// The index within the `&str` at which this line breaks into a new line, along with the
    /// index at which the following line begins. The variant describes whether the break is
    /// caused by a `Newline` character or a `Wrap` by the given wrap function.
    pub end_break: Break,
    /// The total width of all characters within the line.
    pub width: Scalar,
}

impl LineInfo {
    /// The end of the byte index range for indexing into the slice.
    pub fn end_byte(&self) -> usize {
        self.end_break.byte
    }

    /// The end of the index range for indexing into the slice.
    pub fn end_char(&self) -> usize {
        self.end_break.char
    }

    /// The index range for indexing (via bytes) into the original str slice.
    pub fn byte_range(self) -> std::ops::Range<usize> {
        self.start_byte..self.end_byte()
    }

    /// The index range for indexing into a `char` iterator over the original str slice.
    pub fn char_range(self) -> std::ops::Range<usize> {
        self.start_char..self.end_char()
    }
}

/// An iterator yielding an `Info` struct for each line in the given `text` wrapped by the
/// given `next_break_fn`.
///
/// `Infos` is a fundamental part of performing lazy reasoning about text within conrod.
///
/// Construct an `Infos` iterator via the [infos function](./fn.infos.html) and its two builder
/// methods, [wrap_by_character](./struct.Infos.html#method.wrap_by_character) and
/// [wrap_by_whitespace](./struct.Infos.html#method.wrap_by_whitespace).
#[derive(Copy, Clone)]
pub struct LineInfos<'a> {
    text: &'a str,
    font: &'a Font,
    font_size: Scalar,
    max_width: Scalar,
    line_wrap: Wrap,
    /// The index that indicates the start of the next line to be yielded.
    start_byte: usize,
    /// The character index that indicates the start of the next line to be yielded.
    start_char: usize,
    /// The break type of the previously yielded line
    last_break: Option<Break>,
}

impl<'a> LineInfos<'a> {
    pub fn new(text: &'a str,
               font: &'a Font,
               font_size: Scalar,
               line_wrap: Wrap,
               max_width: Scalar)
               -> Self {
        LineInfos {
            text: text,
            font: font,
            font_size: font_size,
            max_width: max_width,
            line_wrap: line_wrap,
            start_byte: 0,
            start_char: 0,
            last_break: None,
        }
    }
}

impl<'a> Iterator for LineInfos<'a> {
    type Item = LineInfo;
    fn next(&mut self) -> Option<Self::Item> {
        let LineInfos { text,
                        font,
                        font_size,
                        max_width,
                        line_wrap,
                        ref mut start_byte,
                        ref mut start_char,
                        ref mut last_break } = *self;

        let text_line = &text[*start_byte..];
        let (next, width) = match line_wrap {
            Wrap::NoWrap => next_break(text_line, font, font_size),
            Wrap::Character => next_break_by_character(text_line, font, font_size, max_width),
            Wrap::Whitespace => next_break_by_whitespace(text_line, font, font_size, max_width),
        };
        match next.break_type {
            BreakType::Newline { len_bytes } |
            BreakType::Wrap { len_bytes } => {
                if next.byte == 0 && len_bytes == 0 {
                    None
                } else {
                    let next_break = Break::new(*start_byte + next.byte,
                                                *start_char + next.char,
                                                next.break_type);
                    let info = LineInfo {
                        start_byte: *start_byte,
                        start_char: *start_char,
                        end_break: next_break,
                        width: width,
                    };
                    *start_byte = info.start_byte + next.byte + len_bytes;
                    *start_char = info.start_char + next.char + 1;
                    *last_break = Some(next_break);
                    Some(info)
                }
            }
            BreakType::End => {
                let char = next.char;
                // if the last line ends in a new line, or the entire text is empty,
                // return an empty line Info
                let empty_line = {
                    match *last_break {
                        Some(last_break_) => {
                            match last_break_.break_type {
                                BreakType::Newline { .. } => true,
                                _ => false,
                            }
                        }
                        None => true,
                    }
                };
                if *start_byte < text.len() || empty_line {
                    let total_bytes = text.len();
                    let total_chars = *start_char + char;
                    let end_break = Break::new(total_bytes, total_chars, BreakType::End);
                    let info = LineInfo {
                        start_byte: *start_byte,
                        start_char: *start_char,
                        end_break: end_break,
                        width: width,
                    };
                    *start_byte = total_bytes;
                    *start_char = total_chars;
                    *last_break = Some(end_break);
                    Some(info)
                } else {
                    None
                }
            }
        }
    }
}

/// An iterator yielding a `Rect` for each line in
#[derive(Clone)]
pub struct LineRects<I> {
    infos: I,
    align: Align,
    line_height: Scalar,
    next: Option<Rectangle>,
}

impl<I> LineRects<I>
    where I: Iterator<Item = LineInfo> + ExactSizeIterator
{
    /// Produce an iterator yielding the bounding `Rect` for each line in the text.
    ///
    /// This function assumes that `font_size` is the same `FontSize` used to produce the `Info`s
    /// yielded by the `infos` Iterator.
    pub fn new(mut infos: I,
               font_size: Scalar,
               bounding_rect: Rectangle,
               align: Align,
               line_height: Scalar)
               -> Self {
        let num_lines = infos.len();
        let first_rect = infos.next().map(|first_info| {
            let bounding_x = bounding_rect.x_range();
            let bounding_y = bounding_rect.y_range();
            // Calculate the `x` `Range` of the first line `Rect`.
            let range = Range::new(0.0, first_info.width);
            let x = match align {
                Align::Start => range.align_start_of(bounding_x),
                Align::Middle => range.align_middle_of(bounding_x),
                Align::End => range.align_end_of(bounding_x),
            };

            // Calculate the `y` `Range` of the first line `Rect`.
            let total_text_height = num_lines as Scalar * line_height;
            let total_text_y_range = Range::new(0.0, total_text_height);
            let total_text_y = total_text_y_range.align_start_of(bounding_y);
            let range = Range::new(0.0, font_size as Scalar);
            let y = range.align_start_of(total_text_y);

            Rectangle::from_ranges(x, y)
        });

        LineRects {
            infos: infos,
            next: first_rect,
            align: align,
            line_height: line_height,
        }
    }
}

impl<I> Iterator for LineRects<I>
    where I: Iterator<Item = LineInfo>
{
    type Item = Rectangle;
    fn next(&mut self) -> Option<Self::Item> {
        let LineRects { ref mut next, ref mut infos, align, line_height } = *self;
        next.map(|line_rect| {
            *next = infos.next().map(|info| {
                let y = Range::new(line_rect.bottom(), line_rect.bottom() + line_height);
                let x = {
                    let range = Range::new(0.0, info.width);
                    match align {
                        Align::Start => range.align_start_of(line_rect.x_range()),
                        Align::Middle => range.align_middle_of(line_rect.x_range()),
                        Align::End => range.align_end_of(line_rect.x_range()),
                    }
                };
                Rectangle::from_ranges(x, y)
            });

            line_rect
        })
    }
}

/// An iterator yielding a `Rect` for each selected line in a block of text.
///
/// The yielded `Rect`s represent the selected range within each line of text.
///
/// Lines that do not contain any selected text will be skipped.
pub struct SelectedLineRects<'a, I> {
    selected_glyph_rects_per_line: super::glyph::SelectedGlyphRectsPerLine<'a, I>,
}
impl<'a, I> SelectedLineRects<'a, I>
    where I: Iterator<Item = (&'a str, Rectangle)>
{
    /// Produces an iterator yielding a `Rect` for the selected range in each
    /// selected line in a block of text.
    ///
    /// The yielded `Rect`s represent the selected range within each line of text.
    ///
    /// Lines that do not contain any selected text will be skipped.
    pub fn new(lines_with_rects: I,
               font: &'a Font,
               font_size: Scalar,
               start: super::cursor::Index,
               end: super::cursor::Index)
               -> SelectedLineRects<'a, I> {
        SelectedLineRects {
            selected_glyph_rects_per_line: SelectedGlyphRectsPerLine::new(lines_with_rects,
                                                                          font,
                                                                          font_size,
                                                                          start,
                                                                          end),
        }
    }
}
impl<'a, I> Iterator for SelectedLineRects<'a, I>
    where I: Iterator<Item = (&'a str, Rectangle)>
{
    type Item = Rectangle;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(mut rects) = self.selected_glyph_rects_per_line.next() {
            if let Some(first_rect) = rects.next() {
                let total_selected_rect = rects.fold(first_rect, |mut total, next| {
                    // TODO ?
                    total.width = next.width;
                    total
                });
                return Some(total_selected_rect);
            }
        }
        None
    }
}

/// A function for finding the advance width between the given character that also considers
/// the kerning for some previous glyph.
///
/// This also updates the `last_glyph` with the glyph produced for the given `char`.
///
/// This is primarily for use within the `next_break` functions below.
///
/// The following code is adapted from the rusttype::LayoutIter::next src.
fn advance_width(ch: char, font: &Font, scale: Scale, last_glyph: &mut Option<GlyphId>) -> Scalar {
    let g = font.glyph(ch).unwrap().scaled(scale);
    let kern = last_glyph.map(|last| font.pair_kerning(scale, last, g.id()))
        .unwrap_or(0.0);
    let advance_width = g.h_metrics().advance_width;
    *last_glyph = Some(g.id());
    (kern + advance_width) as Scalar
}

fn peek_next_char(char_indices: &mut Peekable<CharIndices>, next_char_expected: char) -> bool {
    if let Some(&(_, next_char)) = char_indices.peek() {
        next_char == next_char_expected
    } else {
        false
    }
}

/// Returns the next index at which the text naturally breaks via a newline character,
/// along with the width of the line.
fn next_break(text: &str, font: &Font, font_size: Scalar) -> (Break, Scalar) {
    let scale = super::pt_to_scale(font_size);
    let mut width = 0.0;
    let mut char_i = 0;
    let mut char_indices = text.char_indices().peekable();
    let mut last_glyph = None;
    while let Some((byte_i, ch)) = char_indices.next() {
        // Check for a newline.
        if ch == '\r' && peek_next_char(&mut char_indices, '\n') {
            let break_ = Break::new(byte_i, char_i, BreakType::Newline { len_bytes: 2 });
            return (break_, width);
        } else if ch == '\n' {
            let break_ = Break::new(byte_i, char_i, BreakType::Newline { len_bytes: 1 });
            return (break_, width);
        }

        // Update the width.
        width += advance_width(ch, font, scale, &mut last_glyph);
        char_i += 1;
    }
    let break_ = Break::new(text.len(), char_i, BreakType::End);
    (break_, width)
}
/// Returns the next index at which the text will break by either:
/// - A newline character.
/// - A line wrap at the beginning of the first character exceeding the `max_width`.
///
/// Also returns the width of each line alongside the Break.
fn next_break_by_character(text: &str,
                           font: &Font,
                           font_size: Scalar,
                           max_width: Scalar)
                           -> (Break, Scalar) {
    let scale = super::pt_to_scale(font_size);
    let mut width = 0.0;
    let mut char_i = 0;
    let mut char_indices = text.char_indices().peekable();
    let mut last_glyph = None;
    while let Some((byte_i, ch)) = char_indices.next() {
        // Check for a newline.
        if ch == '\r' && peek_next_char(&mut char_indices, '\n') {
            let break_ = Break::new(byte_i, char_i, BreakType::Newline { len_bytes: 2 });
            return (break_, width);
        } else if ch == '\n' {
            let break_ = Break::new(byte_i, char_i, BreakType::Newline { len_bytes: 1 });
            return (break_, width);
        }

        // Add the character's width to the width so far.
        let new_width = width + advance_width(ch, font, scale, &mut last_glyph);

        // Check for a line wrap.
        if new_width > max_width {
            let break_ = Break::new(byte_i, char_i, BreakType::Wrap { len_bytes: 0 });
            return (break_, width);
        }

        width = new_width;
        char_i += 1;
    }

    let break_ = Break::new(text.len(), char_i, BreakType::End);
    (break_, width)
}

/// Returns the next index at which the text will break by either:
/// - A newline character.
/// - A line wrap at the beginning of the whitespace that preceeds the first word
/// exceeding the `max_width`.
/// - A line wrap at the beginning of the first character exceeding the `max_width`,
/// if no whitespace appears for `max_width` characters.
///
/// Also returns the width the line alongside the Break.
fn next_break_by_whitespace(text: &str,
                            font: &Font,
                            font_size: Scalar,
                            max_width: Scalar)
                            -> (Break, Scalar) {
    struct Last {
        byte: usize,
        char: usize,
        width_before: Scalar,
    }
    let scale = super::pt_to_scale(font_size);
    let mut last_whitespace_start = None;
    let mut width = 0.0;
    let mut char_i = 0;
    let mut char_indices = text.char_indices().peekable();
    let mut last_glyph = None;
    while let Some((byte_i, ch)) = char_indices.next() {

        // Check for a newline.
        if ch == '\r' && peek_next_char(&mut char_indices, '\n') {
            let break_ = Break::new(byte_i, char_i, BreakType::Newline { len_bytes: 2 });
            return (break_, width);
        } else if ch == '\n' {
            let break_ = Break::new(byte_i, char_i, BreakType::Newline { len_bytes: 1 });
            return (break_, width);
        }

        // Add the character's width to the width so far.
        let new_width = width + advance_width(ch, font, scale, &mut last_glyph);

        // Check for a line wrap.
        if new_width > max_width {
            match last_whitespace_start {
                Some(Last { byte, char, width_before }) => {
                    let break_ = Break::new(byte, char, BreakType::Wrap { len_bytes: 1 });
                    return (break_, width_before);
                }
                None => {
                    let break_ = Break::new(byte_i, char_i, BreakType::Wrap { len_bytes: 0 });
                    return (break_, width);
                }
            }
        }

        // Check for a new whitespace.
        if ch.is_whitespace() {
            last_whitespace_start = Some(Last {
                byte: byte_i,
                char: char_i,
                width_before: width,
            });
        }

        width = new_width;
        char_i += 1;
    }

    let break_ = Break::new(text.len(), char_i, BreakType::End);
    (break_, width)
}

/// Produce the width of the given line of text including spaces (i.e. ' ').
pub fn width(text: &str, font: &Font, font_size: Scalar) -> Scalar {
    let scale = Scale::uniform(font_size as f32);
    let point = rusttype::Point { x: 0.0, y: 0.0 };

    let mut total_w = 0.0;
    for g in font.layout(text, scale, point) {
        match g.pixel_bounding_box() {
            Some(bb) => total_w = bb.max.x as f32,
            None => total_w += g.unpositioned().h_metrics().advance_width,
        }
    }
    total_w as Scalar
}
