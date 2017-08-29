use webrender_api::{LocalClip, BorderRadius, ComplexClipRegion};

use render::RenderBuilder;
use widget::drawable::Drawable;
use widget::property::PropSet;
use widget::style::{Value, Styleable};
use util::{Rect, RectExt};
use color::*;

pub struct RectDrawable {
    pub background_color: Color,
    pub corner_radius: Option<f32>,
    pub border: Option<(f32, Color)>,
}
impl Default for RectDrawable {
    fn default() -> Self {
        RectDrawable {
            background_color: WHITE,
            corner_radius: None,
            border: None,
        }
    }
}
impl RectDrawable {
    pub fn new() -> Self {
        RectDrawable::default()
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
impl Drawable for RectDrawable {
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

#[derive(Clone)]
pub enum RectStyleable {
    BackgroundColor(Value<Color>),
    CornerRadius(Value<Option<f32>>),
    Border(Value<Option<(f32, Color)>>),
}

impl Styleable<RectDrawable> for RectStyleable {
    fn apply(&self, drawable: &mut RectDrawable, props: &PropSet) {
        match *self {
            RectStyleable::BackgroundColor(ref val) => {
                drawable.background_color = val.get(props)
            }
            RectStyleable::CornerRadius(ref val) => drawable.corner_radius = val.get(props),
            RectStyleable::Border(ref val) => drawable.border = val.get(props),
        }
    }
}
