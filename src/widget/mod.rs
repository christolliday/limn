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
use self::drawable::Drawable;

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
    pub drawable: Option<Drawable>,
    pub layout: LayoutVars,
    pub debug_name: Option<String>,
    pub debug_color: Option<Color>,
}

impl Widget {
    pub fn new(id: WidgetId,
               drawable: Option<Drawable>,
               layout: LayoutVars,
               debug_name: Option<String>,
               debug_color: Option<Color>)
               -> Self {
        Widget {
            id: id,
            drawable: drawable,
            layout: layout,
            debug_name: debug_name,
            debug_color: debug_color,
        }
    }
    pub fn draw(&mut self,
                crop_to: Rectangle,
                glyph_cache: &mut GlyphCache,
                context: Context,
                graphics: &mut G2d) {

        if let Some(drawable) = self.drawable.as_mut() {
            let bounds = self.layout.bounds();
            drawable.draw(bounds, crop_to, glyph_cache, context, graphics);
        }
    }
    pub fn is_mouse_over(&self, mouse: Point) -> bool {
        let bounds = self.layout.bounds();
        if let Some(ref drawable) = self.drawable {
            if let Some(mouse_over_fn) = drawable.mouse_over_fn {
                return mouse_over_fn(mouse, bounds);
            }
        }
        util::point_inside_rect(mouse, bounds)
    }
}
