use webrender_api::{LocalClip, BorderRadius, ComplexClipRegion, PrimitiveInfo};

use render::RenderBuilder;
use widget::draw::Draw;
use widget::property::PropSet;
use widget::style::{self, Style, Value};
use geometry::{Rect, RectExt};
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

fn clip_rounded(rect: Rect, radius: f32) -> LocalClip {
    let rect = rect.typed();
    let clip_region = ComplexClipRegion::new(rect, BorderRadius::uniform(radius));
    LocalClip::RoundedRect(rect, clip_region)
}

fn push_rect(renderer: &mut RenderBuilder, rect: Rect, color: Color, clip_rect: Rect, radius: Option<f32>) {
    let info = if let Some(radius) = radius {
        PrimitiveInfo::with_clip(rect.typed(), clip_rounded(clip_rect, radius))
    } else {
        PrimitiveInfo::new(rect.typed())
    };
    renderer.builder.push_rect(&info, color.into());
}

impl Draw for RectState {
    fn draw(&mut self, bounds: Rect, _: Rect, renderer: &mut RenderBuilder) {
        // rounding is a hack to prevent bug in webrender that produces artifacts around the corners
        let bounds = bounds.round();
        if let Some((width, color)) = self.border {
            let width = if width < 2.0 { 2.0 } else { width };
            push_rect(renderer, bounds, color, bounds, self.corner_radius);
            push_rect(renderer, bounds, self.background_color, bounds.shrink_bounds(width), self.corner_radius);
        } else {
            push_rect(renderer, bounds, self.background_color, bounds, self.corner_radius);
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
