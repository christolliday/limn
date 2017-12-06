use webrender::api::{ComplexClipRegion, BorderRadius, LocalClip, PrimitiveInfo, ClipMode};

use render::RenderBuilder;
use widget::draw::Draw;
use geometry::{Rect, RectExt, Point, Size};
use color::*;

use webrender::api::*;

component_style!{pub struct EllipseState<name="ellipse", style=EllipseStyle> {
    background_color: Color = BLACK,
    border: Option<(f32, Color)> = None,
    alpha: f32 = 1.0,
}}

impl Draw for EllipseState {
    fn draw(&mut self, bounds: Rect, _: Rect, renderer: &mut RenderBuilder) {
        // rounding is a hack to prevent bug in webrender that produces artifacts around the corners
        let bounds = bounds.round();

        if self.alpha != 1.0 {
            renderer.builder.push_stacking_context(
                &PrimitiveInfo::new(Rect::zero()),
                ScrollPolicy::Fixed,
                None,
                TransformStyle::Flat,
                None,
                MixBlendMode::Normal,
                vec![FilterOp::Opacity(PropertyBinding::Value(self.alpha), self.alpha)],
            );
        }

        if let Some((width, color)) = self.border {
            let width = if width < 2.0 { 2.0 } else { width };
            push_ellipse(renderer, bounds, bounds, color);
            push_ellipse(renderer, bounds, bounds.shrink_bounds(width), self.background_color);
        } else {
            push_ellipse(renderer, bounds, bounds, self.background_color);
        };

        if self.alpha != 1.0 {
            renderer.builder.pop_stacking_context();
        }
    }
    fn is_under_cursor(&self, bounds: Rect, cursor: Point) -> bool {
        let radius = Size::new(bounds.width() / 2.0, bounds.height() / 2.0);
        let center = Point::new(bounds.left() + radius.width, bounds.top() + radius.height);
        point_inside_ellipse(cursor, center, radius)
    }
}

fn clip_ellipse(rect: Rect) -> LocalClip {
    let clip_region = ComplexClipRegion::new(rect, BorderRadius::uniform_size(rect.size / 2.0), ClipMode::Clip);
    LocalClip::RoundedRect(rect, clip_region)
}

fn push_ellipse(renderer: &mut RenderBuilder, rect: Rect, clip_rect: Rect, color: Color) {
    let clip = clip_ellipse(clip_rect);
    let info = PrimitiveInfo::with_clip(rect, clip);
    renderer.builder.push_rect(&info, color.into());
}

fn point_inside_ellipse(point: Point, center: Point, radius: Size) -> bool {
    (point.x - center.x).powi(2) / radius.width.powi(2) +
    (point.y - center.y).powi(2) / radius.height.powi(2) <= 1.0
}
