use std::fmt;
use webrender::api::ColorF;

fn rgba(color: u32) -> ColorF {
    let read = |offset: u32| { (color >> offset & 0xFF) as f32 / 255.0 };
    ColorF::new(read(24), read(16), read(8), read(0))
}

// needed to declare const val colors, when const fns are stable, can use rgba to construct ColorF directly
#[derive(Clone, Copy)]
pub struct Color(u32);

impl ::std::fmt::Debug for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Color(0x{:08X})", self.0)
    }
}

impl Into<ColorF> for Color {
    fn into(self) -> ColorF {
        rgba(self.0)
    }
}

pub const TRANSPARENT: Color = Color(0x00000000);
pub const BLACK: Color = Color(0x000000FF);
pub const WHITE: Color = Color(0xFFFFFFFF);

pub const GRAY_10: Color = Color(0x191919FF);
pub const GRAY_20: Color = Color(0x333333FF);
pub const GRAY_30: Color = Color(0x4C4C4CFF);
pub const GRAY_40: Color = Color(0x666666FF);
pub const GRAY_50: Color = Color(0x808080FF);
pub const GRAY_60: Color = Color(0x999999FF);
pub const GRAY_70: Color = Color(0xB2B2B2FF);
pub const GRAY_80: Color = Color(0xCCCCCCFF);
pub const GRAY_90: Color = Color(0xE5E5E5FF);

pub const RED: Color = Color(0xFF0000FF);
pub const GREEN: Color = Color(0x00FF00FF);
pub const BLUE: Color = Color(0x0000FFFF);

pub const YELLOW: Color = Color(0xFFFF00FF);
pub const FUSCHIA: Color = Color(0xFF00FFFF);
pub const CYAN: Color = Color(0x00FFFFFF);

pub const BLUE_HIGHLIGHT: Color = Color(0x6060D0FF);
