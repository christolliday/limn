use webrender::api::{LocalClip, BorderRadius, ComplexClipRegion, PrimitiveInfo, ClipMode};

use render::RenderBuilder;
use widget::draw::Draw;
use geometry::{Rect, RectExt};
use color::*;

component_style!{pub struct RectState<name="rect", style=RectStyle> {
    background_color: Color = WHITE,
    corner_radius: Option<f32> = None,
    border: Option<(f32, Color)> = None,
}}

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

fn clip_rounded(rect: Rect, radius: f32) -> LocalClip {
    let clip_region = ComplexClipRegion::new(rect, BorderRadius::uniform(radius), ClipMode::Clip);
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
