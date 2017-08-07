use std::any::Any;
use std::marker::PhantomData;

use downcast_rs::Downcast;
use graphics::Context;

use backend::gfx::G2d;
use backend::glyph::GlyphCache;

use event::{WidgetEventHandler, WidgetEventArgs};
use widget::property::PropSet;
use widget::style::Style;

use util::Rect;


pub trait Drawable: Downcast {
    fn draw(&mut self, bounds: Rect, crop_to: Rect, glyph_cache: &mut GlyphCache, context: Context, graphics: &mut G2d);
}
impl_downcast!(Drawable);

type StyleFn = Fn(&mut Drawable, &Any, &PropSet);

pub struct DrawableStyle {
    pub style: Box<Any>,
    pub style_fn: Box<StyleFn>,
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
        let style_fn = |drawable: &mut Drawable, style: &Any, props: &PropSet| {
            let drawable: &mut T = drawable.downcast_mut().unwrap();
            let style: &S = style.downcast_ref().unwrap();
            style.apply(drawable, props);
        };
        let style = Some(DrawableStyle {
            style: Box::new(style),
            style_fn: Box::new(style_fn),
        });
        DrawableWrapper {
            drawable: Box::new(drawable),
            style: style,
        }
    }
    pub fn apply_style(&mut self, props: &PropSet) -> bool {
        if let Some(ref style) = self.style {
            (style.style_fn)(self.drawable.as_mut(), style.style.as_ref(), props);
            true
        } else {
            false
        }
    }
}

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

impl<T: Drawable + 'static, E> WidgetEventHandler<E> for DrawableEventHandler<T, E> {
    fn handle(&mut self, _: &E, mut args: WidgetEventArgs) {
        args.widget.update(|state: &mut T| {
            (self.drawable_callback)(state);
        });
    }
}
