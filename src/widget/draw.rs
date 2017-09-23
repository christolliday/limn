use std::any::Any;
use std::marker::PhantomData;

use downcast_rs::Downcast;

use render::RenderBuilder;
use event::{EventHandler, EventArgs};
use widget::property::PropSet;
use widget::style::Style;

use util::{Rect, Point};


pub trait Draw: Downcast {
    fn draw(&mut self, bounds: Rect, crop_to: Rect, renderer: &mut RenderBuilder);
    fn is_under_cursor(&self, bounds: Rect, cursor: Point) -> bool {
        bounds.contains(&cursor)
    }
}
impl_downcast!(Draw);

type StyleFn = Fn(&mut Draw, &Any, &PropSet) -> bool;

pub(super) struct DrawStyle {
    pub style: Box<Any>,
    pub style_fn: Box<StyleFn>,
}
pub(super) struct DrawWrapper {
    pub state: Box<Draw>,
    pub style: Option<DrawStyle>,
}
impl DrawWrapper {
    pub fn new<T: Draw + 'static>(draw_state: T) -> Self
    {
        DrawWrapper {
            state: Box::new(draw_state),
            style: None,
        }
    }
    pub fn new_with_style<T: Draw + 'static, S: Style<T> + 'static>(draw_state: T, style: S) -> Self
    {
        let style_fn = |draw_state: &mut Draw, style: &Any, props: &PropSet| -> bool {
            let draw_state: &mut T = draw_state.downcast_mut().unwrap();
            let style: &S = style.downcast_ref().unwrap();
            style.apply(draw_state, props)
        };
        let style = Some(DrawStyle {
            style: Box::new(style),
            style_fn: Box::new(style_fn),
        });
        DrawWrapper {
            state: Box::new(draw_state),
            style: style,
        }
    }
    pub fn apply_style(&mut self, props: &PropSet) -> bool {
        if let Some(ref style) = self.style {
            (style.style_fn)(self.state.as_mut(), style.style.as_ref(), props)
        } else {
            false
        }
    }
    pub fn is_under_cursor(&self, bounds: Rect, cursor: Point) -> bool {
        self.state.is_under_cursor(bounds, cursor)
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
