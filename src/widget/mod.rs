pub mod layout;
pub mod builder;
pub mod style;
pub mod property;
pub mod drawable;

use std::any::{TypeId, Any};

use graphics::Context;
use graphics::types::Color;

use backend::gfx::G2d;
use backend::glyph::GlyphCache;

use ui::queue::EventQueue;
use resources::WidgetId;
use ui::layout::LimnSolver;
use util::{self, Point, Rectangle};

use self::property::PropSet;
use self::layout::LayoutVars;
use self::drawable::{Drawable, DrawableWrapper};

pub use self::builder::WidgetBuilder;

// allows event handlers to communicate with event dispatcher
pub struct EventState {
    pub handled: bool,
}

pub struct HandlerWrapper {
    type_id: TypeId,
    handler: Box<Any>,
    handle_fn: Box<Fn(&mut Box<Any>, &Box<Any + Send>, EventArgs)>,
}
impl HandlerWrapper {
    pub fn new<H, E>(handler: H) -> Self
        where H: EventHandler<E> + 'static,
              E: 'static
    {
        let handle_fn = |handler: &mut Box<Any>, event: &Box<Any + Send>, args: EventArgs| {
            let event: &E = event.downcast_ref().unwrap();
            let handler: &mut H = handler.downcast_mut().unwrap();
            handler.handle(event, args);
        };
        HandlerWrapper {
            type_id: TypeId::of::<E>(),
            handler: Box::new(handler),
            handle_fn: Box::new(handle_fn),
        }
    }
    pub fn handles(&self, type_id: TypeId) -> bool {
        self.type_id == type_id
    }
    pub fn handle(&mut self, event: &Box<Any + Send>, args: EventArgs) {
        (self.handle_fn)(&mut self.handler, event, args);
    }
}

pub struct EventArgs<'a> {
    pub widget: &'a mut Widget,
    pub event_queue: &'a mut EventQueue,
    pub solver: &'a mut LimnSolver,
    pub event_state: &'a mut EventState,
}

pub trait EventHandler<T> {
    fn handle(&mut self, event: &T, args: EventArgs);
}

pub struct WidgetContainer {
    pub widget: Widget,
    pub event_handlers: Vec<HandlerWrapper>,
}
impl WidgetContainer {
    pub fn trigger_event(&mut self,
                         type_id: TypeId,
                         event: &Box<Any + Send>,
                         event_queue: &mut EventQueue,
                         solver: &mut LimnSolver)
                         -> bool {

        let mut event_state = EventState { handled: false };
        for ref mut event_handler in self.event_handlers.iter_mut() {
            let event_handler: &mut HandlerWrapper = event_handler;
            if event_handler.handles(type_id) {
                let event_args = EventArgs {
                    widget: &mut self.widget,
                    event_queue: event_queue,
                    solver: solver,
                    event_state: &mut event_state,
                };
                event_handler.handle(event, event_args);
            }
        }
        event_state.handled
    }
}
pub struct Widget {
    pub id: WidgetId,
    pub drawable: Option<DrawableWrapper>,
    pub props: PropSet,
    pub has_updated: bool,
    pub layout: LayoutVars,
    pub debug_name: Option<String>,
    pub debug_color: Option<Color>,
}

impl Widget {
    pub fn new(id: WidgetId,
               drawable: Option<DrawableWrapper>,
               props: PropSet,
               layout: LayoutVars,
               debug_name: Option<String>,
               debug_color: Option<Color>)
               -> Self {
        let mut widget = Widget {
            id: id,
            drawable: drawable,
            props: props,
            has_updated: false,
            layout: layout,
            debug_name: debug_name,
            debug_color: debug_color,
        };
        widget.apply_style();
        widget
    }
    pub fn draw(&mut self,
                crop_to: Rectangle,
                glyph_cache: &mut GlyphCache,
                context: Context,
                graphics: &mut G2d) {

        if let Some(drawable) = self.drawable.as_mut() {
            let bounds = self.layout.bounds();
            let context = util::crop_context(context, crop_to);
            drawable.drawable.draw(bounds, crop_to, glyph_cache, context, graphics);
        }
    }

    pub fn is_mouse_over(&self, mouse: Point) -> bool {
        util::point_inside_rect(mouse, self.layout.bounds())
    }

    pub fn update<F, T: Drawable + 'static>(&mut self, f: F)
        where F: FnOnce(&mut T)
    {
        if let Some(ref mut drawable) = self.drawable {
            self.has_updated = true;
            let state = drawable.drawable.as_mut().downcast_mut::<T>().unwrap();
            f(state);
        }
    }

    pub fn apply_style(&mut self) {
        if let Some(ref mut drawable) = self.drawable {
            if drawable.apply_style(&self.props) {
                self.has_updated = true;
            }
        }
    }
}
