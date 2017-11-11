use std::any::Any;
use std::marker::PhantomData;

use render::RenderBuilder;
use event::{EventHandler, EventArgs};
use widget::property::PropSet;
use widget::style::PropSelector;
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
    pub fn new<D, T: IntoDrawState<D> + 'static>(draw_state: T) -> Self {
        DrawWrapper {
            wrapper: draw_state.draw_state()
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

pub trait IntoDrawState<D> {
    fn draw_state(self) -> Box<DrawComponent>;
}

impl <D: Draw + DrawComponent + 'static> IntoDrawState<Box<DrawComponent>> for D {
    fn draw_state(self) -> Box<DrawComponent> {
        Box::new(self)
    }
}

impl <P: PropSelector<D> + 'static, D: Draw + Default + 'static> IntoDrawState<Box<PropDrawWrapper<D>>> for P {
    fn draw_state(self) -> Box<DrawComponent> {
        Box::new(PropDrawWrapper::new(self))
    }
}

pub struct PropDrawWrapper<D: Draw> {
    state: D,
    prop_selector: Box<PropSelector<D>>,
}

impl <D: Draw + Default + 'static> PropDrawWrapper<D> {
    pub fn new<T: PropSelector<D> + 'static>(prop_selector: T) -> Self {
        PropDrawWrapper {
            state: D::default(),
            prop_selector: Box::new(prop_selector),
        }
    }
}

impl <D: Draw + Default + 'static> DrawComponent for PropDrawWrapper<D> {
    fn state(&self) -> &Any {
        &self.state
    }
    fn state_mut(&mut self) -> &mut Any {
        &mut self.state
    }
    fn apply_style(&mut self, props: &PropSet) -> bool {
        self.prop_selector.as_ref().apply(&mut self.state, props)
    }
}

impl <D: Draw> Draw for PropDrawWrapper<D> {
    fn is_under_cursor(&self, bounds: Rect, cursor: Point) -> bool {
        self.state.is_under_cursor(bounds, cursor)
    }
    fn draw(&mut self, bounds: Rect, crop_to: Rect, renderer: &mut RenderBuilder) {
        self.state.draw(bounds, crop_to, renderer);
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
