use std::marker::PhantomData;

use mopa;
use webrender::api::*;

use render::RenderBuilder;
use event::{EventHandler, EventArgs};

use geometry::Rect;


pub trait Draw: ::std::fmt::Debug + mopa::Any {
    fn draw(&mut self, bounds: Rect, crop_to: Rect, renderer: &mut RenderBuilder);
}

mopafy!(Draw);

pub struct DrawEventHandler<T, E> {
    draw_callback: Box<Fn(&mut T)>,
    phantom: PhantomData<E>,
}

impl<T: 'static, E> DrawEventHandler<T, E> {
    pub fn new<F: Fn(&mut T) + 'static>(_: E, draw_callback: F) -> Self {
        DrawEventHandler {
            draw_callback: Box::new(draw_callback),
            phantom: PhantomData,
        }
    }
}

impl<T: Draw + 'static, E> EventHandler<E> for DrawEventHandler<T, E> {
    fn handle(&mut self, _: &E, mut args: EventArgs) {
        args.widget.update(|state: &mut T| {
            (self.draw_callback)(state);
        });
    }
}

pub trait DrawModifier: mopa::Any {
    fn push(&self, renderer: &mut RenderBuilder);
    fn pop(&self, renderer: &mut RenderBuilder);
}

mopafy!(DrawModifier);

pub struct OpacityModifier {
    pub alpha: f32,
}

impl Default for OpacityModifier {
    fn default() -> Self {
        OpacityModifier {
            alpha: 1.0,
        }
    }
}

impl DrawModifier for OpacityModifier {
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
