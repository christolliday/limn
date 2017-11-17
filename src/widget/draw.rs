use std::any::Any;
use std::marker::PhantomData;

use render::RenderBuilder;
use event::{EventHandler, EventArgs};
use widget::property::PropSet;
use style::Component;

use geometry::{Rect, Point};


pub trait Draw {
    fn draw(&mut self, bounds: Rect, crop_to: Rect, renderer: &mut RenderBuilder);
    fn is_under_cursor(&self, bounds: Rect, cursor: Point) -> bool {
        bounds.contains(&cursor)
    }
}

pub trait DrawComponent: Draw {
    fn state(&self) -> &Any;
    fn state_mut(&mut self) -> &mut Any;
    fn apply_style(&mut self, props: &PropSet) -> bool;
}

pub struct DrawWrapper {
    pub wrapper: Box<DrawComponent>,
}

impl DrawWrapper {
    pub fn new<T: Draw + Component + 'static>(draw_state: T) -> Self {
        DrawWrapper {
            wrapper: Box::new(draw_state)
        }
    }
}

impl Draw for DrawWrapper {
    fn is_under_cursor(&self, bounds: Rect, cursor: Point) -> bool {
        self.wrapper.is_under_cursor(bounds, cursor)
    }
    fn draw(&mut self, bounds: Rect, crop_to: Rect, renderer: &mut RenderBuilder) {
        self.wrapper.draw(bounds, crop_to, renderer);
    }
}

impl <D: Draw + Component + 'static> DrawComponent for D {
    fn state(&self) -> &Any {
        self
    }
    fn state_mut(&mut self) -> &mut Any {
        self
    }
    fn apply_style(&mut self, _: &PropSet) -> bool {
        false
    }
}

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
