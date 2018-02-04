use std::marker::PhantomData;

use mopa;

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
