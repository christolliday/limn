use webrender_api::{ComplexClipRegion, BorderRadius, LocalClip};

use render::RenderBuilder;
use widget::draw::Draw;
use widget::property::PropSet;
use widget::style::{Style, Value};
use util::{Rect, RectExt, Point, Size};
use color::*;

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

fn clip_ellipse(rect: Rect) -> LocalClip {
    let rect = rect.typed();
    let clip_region = ComplexClipRegion::new(rect, BorderRadius::uniform_size(rect.size / 2.0));
    LocalClip::RoundedRect(rect, clip_region)
}

impl Draw for EllipseState {
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

#[derive(Clone)]
pub enum EllipseStyle {
    BackgroundColor(Value<Color>),
    Border(Value<Option<(f32, Color)>>),
}

impl Style<EllipseState> for EllipseStyle {
    fn apply(&self, draw_state: &mut EllipseState, props: &PropSet) {
        match *self {
            EllipseStyle::BackgroundColor(ref val) => {
                draw_state.background_color = val.get(props)
            },
            EllipseStyle::Border(ref val) => draw_state.border = val.get(props),
        }
    }
}
