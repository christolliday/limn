use mopa;
use webrender::api::*;

use render::RenderBuilder;
use geometry::Rect;

pub trait Filter: mopa::Any {
    fn push(&self, renderer: &mut RenderBuilder);
    fn pop(&self, renderer: &mut RenderBuilder);
}

mopafy!(Filter);

pub struct OpacityFilter {
    pub alpha: f32,
}

impl Default for OpacityFilter {
    fn default() -> Self {
        OpacityFilter {
            alpha: 1.0,
        }
    }
}

impl Filter for OpacityFilter {
    fn push(&self, renderer: &mut RenderBuilder) {
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
    }
    fn pop(&self, renderer: &mut RenderBuilder) {
        if self.alpha != 1.0 {
            renderer.builder.pop_stacking_context();
        }
    }
}
