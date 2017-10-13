use webrender::api::{ComplexClipRegion, BorderRadius, LocalClip, PrimitiveInfo};

use render::RenderBuilder;
use widget::draw::Draw;
use widget::property::PropSet;
use widget::style::{self, Style, Value};
use geometry::{Rect, RectExt, Point, Size};
use color::*;

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

fn clip_ellipse(rect: Rect) -> LocalClip {
    let rect = rect.typed();
    let clip_region = ComplexClipRegion::new(rect, BorderRadius::uniform_size(rect.size / 2.0));
    LocalClip::RoundedRect(rect, clip_region)
}

fn push_ellipse(renderer: &mut RenderBuilder, rect: Rect, clip_rect: Rect, color: Color) {
    let clip = clip_ellipse(clip_rect);
    let info = PrimitiveInfo::with_clip(rect.typed(), clip);
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

#[derive(Clone, Debug)]
pub enum EllipseStyle {
    BackgroundColor(Value<Color>),
    Border(Value<Option<(f32, Color)>>),
}

impl Style<EllipseState> for EllipseStyle {
    fn apply(&self, state: &mut EllipseState, props: &PropSet) -> bool {
        match *self {
            EllipseStyle::BackgroundColor(ref val) => {
                style::update(&mut state.background_color, val.get(props))
            },
            EllipseStyle::Border(ref val) => style::update(&mut state.border, val.get(props)),
        }
    }
}
