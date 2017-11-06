use webrender::api::{LocalClip, BorderRadius, ComplexClipRegion, PrimitiveInfo};

use render::RenderBuilder;
use widget::draw::Draw;
use widget::property::PropSet;
use widget::style::{self, PropSelector, Value};
use geometry::{Rect, RectExt};
use color::*;
use style::*;

#[derive(Debug, Copy, Clone)]
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
    let clip_region = ComplexClipRegion::new(rect, BorderRadius::uniform(radius));
    LocalClip::RoundedRect(rect, clip_region)
}

fn push_rect(renderer: &mut RenderBuilder, rect: Rect, color: Color, clip_rect: Rect, radius: Option<f32>) {
    let info = if let Some(radius) = radius {
        PrimitiveInfo::with_clip(rect, clip_rounded(clip_rect, radius))
    } else {
        PrimitiveInfo::new(rect)
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

#[derive(Default, Clone)]
pub struct RectComponentStyle {
    pub background_color: Option<Value<Color>>,
    pub corner_radius: Option<Value<Option<f32>>>,
    pub border: Option<Value<Option<(f32, Color)>>>,
}

impl ComponentStyle for RectComponentStyle {
    type Component = RectComponent;
    fn merge(&self, other: &Self) -> Self {
        RectComponentStyle {
            background_color: self.background_color.as_ref().or(other.background_color.as_ref()).cloned(),
            corner_radius: self.corner_radius.as_ref().or(other.corner_radius.as_ref()).cloned(),
            border: self.border.as_ref().or(other.border.as_ref()).cloned(),
        }
    }
    fn component(self) -> Self::Component {
        RectComponent {
            background_color: self.background_color.unwrap_or(Value::from(BLACK)),
            corner_radius: self.corner_radius.unwrap_or(Value::from(None)),
            border: self.border.unwrap_or(Value::from(None)),
        }
    }
}

#[derive(Clone)]
pub struct RectComponent {
    pub background_color: Value<Color>,
    pub corner_radius: Value<Option<f32>>,
    pub border: Value<Option<(f32, Color)>>,
}

impl Component for RectComponent {
    fn name() -> String {
        "rect".to_owned()
    }
}

impl PropSelector<RectState> for RectComponent {
    fn apply(&self, state: &mut RectState, props: &PropSet) -> bool {
        style::update(&mut state.background_color, self.background_color.get(props)) |
        style::update(&mut state.corner_radius, self.corner_radius.get(props)) |
        style::update(&mut state.border, self.border.get(props))
    }
}
