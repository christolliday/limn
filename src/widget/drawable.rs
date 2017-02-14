use std::any::Any;

use graphics::Context;

use backend::gfx::G2d;
use backend::glyph::GlyphCache;

use widget::{EventHandler, EventArgs};
use widget::property::PropSet;
use event::EventId;

use util::{Point, Rectangle};

pub struct DrawArgs<'a, 'b: 'a> {
    pub state: &'a Any,
    pub bounds: Rectangle,
    pub parent_bounds: Rectangle,
    pub glyph_cache: &'a mut GlyphCache,
    pub context: Context,
    pub graphics: &'a mut G2d<'b>,
}

pub struct DrawArgs2<'a, 'b: 'a> {
    pub bounds: Rectangle,
    pub parent_bounds: Rectangle,
    pub glyph_cache: &'a mut GlyphCache,
    pub context: Context,
    pub graphics: &'a mut G2d<'b>,
}

pub struct Drawable {
    state: Box<Any>,
    pub draw_fn: Box<Fn(DrawArgs)>,
    pub mouse_over_fn: Option<fn(Point, Rectangle) -> bool>,
    pub style: Option<DrawableStyle>,
    pub props: PropSet,
    pub has_updated: bool,
}

impl Drawable {
    pub fn new2<T: Any>(state: T, draw_fn: fn(&T, DrawArgs2)) -> Drawable {
        let draw_fn = move |args: DrawArgs| {
            let DrawArgs { state, bounds, parent_bounds, glyph_cache, context, graphics, .. } = args;
            let state: &T = state.downcast_ref().unwrap();
            draw_fn(state, DrawArgs2 {
                bounds: bounds,
                parent_bounds: parent_bounds,
                glyph_cache: glyph_cache,
                context: context,
                graphics: graphics,
            });
        };
        Drawable {
            state: Box::new(state),
            draw_fn: Box::new(draw_fn),
            mouse_over_fn: None,
            style: None,
            has_updated: false,
            props: PropSet::new(),
        }
    }
    pub fn new<T: Any>(state: T, draw_fn: fn(DrawArgs)) -> Drawable {
        Drawable {
            state: Box::new(state),
            draw_fn: Box::new(draw_fn),
            mouse_over_fn: None,
            style: None,
            has_updated: false,
            props: PropSet::new(),
        }
    }
    pub fn apply_style(&mut self) {
        if let Some(ref style) = self.style {
            (style.style_fn)(StyleArgs {
                state: self.state.as_mut(),
                style: style.style.as_ref(),
                props: &self.props,
            });
            self.has_updated = true;
        }
    }
    pub fn update<F, T: 'static>(&mut self, f: F)
        where F: FnOnce(&mut T)
    {
        self.has_updated = true;
        let state = self.state.as_mut().downcast_mut::<T>().unwrap();
        f(state);
    }
    pub fn state<T: 'static>(&self) -> &T {
        self.state.as_ref().downcast_ref::<T>().unwrap()
    }
    pub fn state_any(&self) -> &Any {
        self.state.as_ref()
    }
}

pub struct StyleArgs<'a> {
    pub state: &'a mut Any,
    pub style: &'a Any,
    pub props: &'a PropSet,
}

pub struct DrawableStyle {
    pub style: Box<Any>,
    pub style_fn: fn(StyleArgs),
}

impl DrawableStyle {
    pub fn new<T: Any>(style: T, style_fn: fn(StyleArgs)) -> Self {
        DrawableStyle {
            style: Box::new(style),
            style_fn: style_fn,
        }
    }
}

pub struct DrawableEventHandler<T> {
    event_id: EventId,
    drawable_callback: Box<Fn(&mut T)>,
}

impl<T: 'static> DrawableEventHandler<T> {
    pub fn new<F: Fn(&mut T) + 'static>(event_id: EventId, drawable_callback: F) -> Self {
        DrawableEventHandler {
            event_id: event_id,
            drawable_callback: Box::new(drawable_callback),
        }
    }
}

impl<T: 'static> EventHandler for DrawableEventHandler<T> {
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn handle_event(&mut self, args: EventArgs) {
        if let Some(drawable) = args.drawable.as_mut() {
            drawable.update(|state: &mut T| (self.drawable_callback)(state));
        }
    }
}
