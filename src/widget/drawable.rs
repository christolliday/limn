use std::any::{TypeId, Any};
use std::marker::PhantomData;

use downcast_rs::Downcast;
use graphics::Context;

use backend::gfx::G2d;
use backend::glyph::GlyphCache;

use widget::{EventHandler, EventArgs};
use widget::property::PropSet;
use widget::style::{self, Style, StyleField};

use util::{self, Point, Rectangle};

pub struct DrawArgs<'a, 'b: 'a, T: 'static> {
    pub state: &'a T,
    pub bounds: Rectangle,
    pub parent_bounds: Rectangle,
    pub glyph_cache: &'a mut GlyphCache,
    pub context: Context,
    pub graphics: &'a mut G2d<'b>,
}

pub struct StyleArgs<'a> {
    pub drawable: &'a mut Drawable,
    pub style: &'a Box<Any>,
    pub props: &'a PropSet,
}

pub struct DrawableStyle {
    pub style: Box<Any>,
    pub style_fn: Box<Fn(StyleArgs)>,
}
impl DrawableStyle {
    pub fn new<D: Drawable, T: Style<D> + 'static>(style: T, style_fn: Box<Fn(StyleArgs)>) -> Self {
        DrawableStyle {
            style: Box::new(style),
            style_fn: style_fn,
        }
    }
}
pub struct DrawableWrapper {
    pub drawable: Box<Drawable>,
    pub style: Option<DrawableStyle>,
}
impl DrawableWrapper {
    pub fn new<T: Drawable + 'static>(drawable: T) -> Self
    {
        DrawableWrapper {
            drawable: Box::new(drawable),
            style: None,
        }
    }
    pub fn new_with_style<T: Drawable + 'static, S: Style<T> + 'static>(drawable: T, style: S) -> Self
    {
        let style_fn = |args: StyleArgs| {
            let StyleArgs { drawable, style, props } = args;
            let drawable: &mut T = drawable.downcast_mut().unwrap();
            let style: &S = style.downcast_ref().unwrap();
            style.apply(drawable, props);
        };
        let style = Some(DrawableStyle::new(style, Box::new(style_fn)));
        DrawableWrapper {
            drawable: Box::new(drawable),
            style: style,
        }
    }
    pub fn apply_style(&mut self, props: &PropSet) -> bool {
        if let Some(ref style) = self.style {
            (style.style_fn)(StyleArgs {
                drawable: self.drawable.as_mut(),
                style: &style.style,
                props: props,
            });
            true
        } else {
            false
        }
    }
}

pub trait Drawable: Downcast {
    fn draw(&mut self, bounds: Rectangle, crop_to: Rectangle, glyph_cache: &mut GlyphCache, context: Context, graphics: &mut G2d);
}
impl_downcast!(Drawable);

pub struct DrawableEventHandler<T, E> {
    drawable_callback: Box<Fn(&mut T)>,
    phantom: PhantomData<E>,
}

impl<T: 'static, E> DrawableEventHandler<T, E> {
    pub fn new<F: Fn(&mut T) + 'static>(_: E, drawable_callback: F) -> Self {
        DrawableEventHandler {
            drawable_callback: Box::new(drawable_callback),
            phantom: PhantomData,
        }
    }
}

impl<T: Drawable + 'static, E> EventHandler<E> for DrawableEventHandler<T, E> {
    fn handle(&mut self, _: &E, args: EventArgs) {
        args.widget.update(|state: &mut T| {
            (self.drawable_callback)(state);
        });
    }
}
