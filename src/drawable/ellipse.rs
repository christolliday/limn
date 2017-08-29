use webrender_api::{ComplexClipRegion, BorderRadius, LocalClip};

use render::RenderBuilder;
use widget::drawable::Drawable;
use widget::property::PropSet;
use widget::style::{Styleable, Value};
use util::{Rect, RectExt};
use color::*;

pub struct EllipseDrawable {
    pub background_color: Color,
    pub border: Option<(f32, Color)>,
}
impl Default for EllipseDrawable {
    fn default() -> Self {
        EllipseDrawable {
            background_color: WHITE,
            border: None,
        }
    }
}

impl EllipseDrawable {
    pub fn new() -> Self {
        EllipseDrawable::default()
    }
}

fn clip_ellipse(rect: Rect) -> LocalClip {
    let rect = rect.typed();
    let clip_region = ComplexClipRegion::new(rect, BorderRadius::uniform_size(rect.size / 2.0));
    LocalClip::RoundedRect(rect, clip_region)
}

impl Drawable for EllipseDrawable {
    fn draw(&mut self, bounds: Rect, _: Rect, renderer: &mut RenderBuilder) {
        // rounding is a hack to prevent bug in webrender that produces artifacts around the corners
        let bounds = bounds.round();
        let outer_clip = clip_ellipse(bounds);
        if let Some((width, color)) = self.border {
            let width = if width < 2.0 { 2.0 } else { width };
            renderer.builder.push_rect(bounds.typed(), Some(outer_clip), color.into());
            let inner_clip = clip_ellipse(bounds.shrink_bounds(width));
            renderer.builder.push_rect(bounds.typed(), Some(inner_clip), self.background_color.into());
        } else {
            renderer.builder.push_rect(bounds.typed(), Some(outer_clip), self.background_color.into());
        };
    }
}


#[derive(Clone)]
pub enum EllipseStyleable {
    BackgroundColor(Value<Color>),
    Border(Value<Option<(f32, Color)>>),
}

impl Styleable<EllipseDrawable> for EllipseStyleable {
    fn apply(&self, drawable: &mut EllipseDrawable, props: &PropSet) {
        match *self {
            EllipseStyleable::BackgroundColor(ref val) => {
                drawable.background_color = val.get(props)
            },
            EllipseStyleable::Border(ref val) => drawable.border = val.get(props),
        }
    }
}
