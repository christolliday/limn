use webrender::api::{ComplexClipRegion, BorderRadius, LocalClip, PrimitiveInfo};

use render::RenderBuilder;
use widget::draw::Draw;
use geometry::{Rect, RectExt, Point, Size};
use color::*;
use style::*;

#[derive(Debug, Copy, Clone)]
pub struct EllipseState {
    pub background_color: Color,
    pub border: Option<(f32, Color)>,
}

impl Default for EllipseState {
    fn default() -> Self {
        EllipseState {
            background_color: WHITE,
            border: None,
        }
    }
}

impl EllipseState {
    pub fn new() -> Self {
        EllipseState::default()
    }
}

impl Component for EllipseState {
    fn name() -> String {
        "ellipse".to_owned()
    }
}

fn clip_ellipse(rect: Rect) -> LocalClip {
    let clip_region = ComplexClipRegion::new(rect, BorderRadius::uniform_size(rect.size / 2.0));
    LocalClip::RoundedRect(rect, clip_region)
}

fn push_ellipse(renderer: &mut RenderBuilder, rect: Rect, clip_rect: Rect, color: Color) {
    let clip = clip_ellipse(clip_rect);
    let info = PrimitiveInfo::with_clip(rect, clip);
    renderer.builder.push_rect(&info, color.into());
}

impl Draw for EllipseState {
    fn draw(&mut self, bounds: Rect, _: Rect, renderer: &mut RenderBuilder) {
        // rounding is a hack to prevent bug in webrender that produces artifacts around the corners
        let bounds = bounds.round();
        if let Some((width, color)) = self.border {
            let width = if width < 2.0 { 2.0 } else { width };
            push_ellipse(renderer, bounds, bounds, color);
            push_ellipse(renderer, bounds, bounds.shrink_bounds(width), self.background_color);
        } else {
            push_ellipse(renderer, bounds, bounds, self.background_color);
        };
    }
    fn is_under_cursor(&self, bounds: Rect, cursor: Point) -> bool {
        let radius = Size::new(bounds.width() / 2.0, bounds.height() / 2.0);
        let center = Point::new(bounds.left() + radius.width, bounds.top() + radius.height);
        point_inside_ellipse(cursor, center, radius)
    }
}

fn point_inside_ellipse(point: Point, center: Point, radius: Size) -> bool {
    (point.x - center.x).powi(2) / radius.width.powi(2) +
    (point.y - center.y).powi(2) / radius.height.powi(2) <= 1.0
}

#[derive(Default, Copy, Clone, Debug)]
pub struct EllipseComponentStyle {
    pub background_color: Option<Color>,
    pub border: Option<Option<(f32, Color)>>,
}

impl ComponentStyle for EllipseComponentStyle {
    type Component = EllipseState;
    fn merge(&self, other: &Self) -> Self {
        EllipseComponentStyle {
            background_color: self.background_color.as_ref().or(other.background_color.as_ref()).cloned(),
            border: self.border.as_ref().or(other.border.as_ref()).cloned(),
        }
    }
    fn component(self) -> Self::Component {
        EllipseState {
            background_color: self.background_color.unwrap_or(BLACK),
            border: self.border.unwrap_or(None),
        }
    }
}
