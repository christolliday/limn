use webrender_api::{LocalClip, BorderRadius, ComplexClipRegion};

use render::RenderBuilder;
use widget::draw::Draw;
use widget::property::PropSet;
use widget::style::{self, Style, Value};
use util::{Rect, RectExt};
use color::*;

pub struct RectState {
    pub background_color: Color,
    pub corner_radius: Option<f32>,
    pub border: Option<(f32, Color)>,
}
impl Default for RectState {
    fn default() -> Self {
        RectState {
            background_color: WHITE,
            corner_radius: None,
            border: None,
        }
    }
}
impl RectState {
    pub fn new() -> Self {
        RectState::default()
    }
}
fn clip_rounded(rect: Rect, radius: Option<f32>) -> Option<LocalClip> {
    if let Some(radius) = radius {
        let rect = rect.typed();
        let clip_region = ComplexClipRegion::new(rect, BorderRadius::uniform(radius));
        Some(LocalClip::RoundedRect(rect, clip_region))
    } else {
        None
    }
}
impl Draw for RectState {
    fn draw(&mut self, bounds: Rect, _: Rect, renderer: &mut RenderBuilder) {
        // rounding is a hack to prevent bug in webrender that produces artifacts around the corners
        let bounds = bounds.round();
        let outer_clip = clip_rounded(bounds, self.corner_radius);
        if let Some((width, color)) = self.border {
            let width = if width < 2.0 { 2.0 } else { width };
            renderer.builder.push_rect(bounds.typed(), outer_clip, color.into());
            let inner_clip = clip_rounded(bounds.shrink_bounds(width), self.corner_radius);
            renderer.builder.push_rect(bounds.typed(), inner_clip, self.background_color.into());
        } else {
            renderer.builder.push_rect(bounds.typed(), outer_clip, self.background_color.into());
        };
    }
}

#[derive(Clone, Debug)]
pub enum RectStyle {
    BackgroundColor(Value<Color>),
    CornerRadius(Value<Option<f32>>),
    Border(Value<Option<(f32, Color)>>),
}

impl Style<RectState> for RectStyle {
    fn apply(&self, state: &mut RectState, props: &PropSet) -> bool {
        match *self {
            RectStyle::BackgroundColor(ref val) => {
                style::update(&mut state.background_color, val.get(props))
            }
            RectStyle::CornerRadius(ref val) => style::update(&mut state.corner_radius, val.get(props)),
            RectStyle::Border(ref val) => style::update(&mut state.border, val.get(props)),
        }
    }
}
