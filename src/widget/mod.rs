pub mod layout;
pub mod builder;

use std::any::Any;

use graphics::Context;
use graphics::types::Color;
use cassowary::Solver;

use backend::gfx::G2d;
use backend::glyph::GlyphCache;

use event::{Event, EventQueue};
use input::EventId;
use resources::Id;
use util::{self, Point, Rectangle};

use self::builder::WidgetBuilder;
use self::layout::WidgetLayout;

pub struct DrawArgs<'a, 'b: 'a> {
    pub state: &'a Any,
    pub bounds: Rectangle,
    pub parent_bounds: Rectangle,
    pub glyph_cache: &'a mut GlyphCache,
    pub context: Context,
    pub graphics: &'a mut G2d<'b>,
}

pub struct EventArgs<'a> {
    pub event: &'a (Event + 'static),
    pub widget_id: Id,
    pub state: &'a mut WidgetState,
    pub layout: &'a mut WidgetLayout,
    pub event_queue: &'a mut EventQueue,
    pub solver: &'a mut Solver,
}

pub trait EventHandler {
    fn event_id(&self) -> EventId;
    fn handle_event(&mut self, mut event_args: EventArgs);
}

pub struct WidgetState {
    pub state: Option<Box<Any>>,
}
impl WidgetState {
    pub fn state<T: 'static>(&mut self) -> &mut T {
        self.state.as_mut().map(|draw| draw.as_mut()).unwrap().downcast_mut::<T>().unwrap()
    }
}

pub struct Widget {
    pub id: Id,
    pub draw_fn: Option<fn(DrawArgs)>,
    pub drawable: WidgetState,
    pub mouse_over_fn: fn(Point, Rectangle) -> bool,
    pub layout: WidgetLayout,
    pub event_handlers: Vec<Box<EventHandler>>,
    pub debug_name: Option<String>,
    pub debug_color: Option<Color>,
}

impl Widget {
    pub fn new(id: Id,
               draw_fn: Option<fn(DrawArgs)>,
               drawable: WidgetState,
               mouse_over_fn: fn(Point, Rectangle) -> bool,
               layout: WidgetLayout,
               event_handlers: Vec<Box<EventHandler>>,
               debug_name: Option<String>,
               debug_color: Option<Color>,
               ) -> Self {
        Widget {
            id: id,
            draw_fn: draw_fn,
            drawable: drawable,
            mouse_over_fn: mouse_over_fn,
            layout: layout,
            event_handlers: event_handlers,
            debug_name: debug_name,
            debug_color: debug_color,
        }
    }
    pub fn draw(&self,
                crop_to: Rectangle,
                solver: &mut Solver,
                glyph_cache: &mut GlyphCache,
                context: Context,
                graphics: &mut G2d) {
        if let (Some(draw_fn), Some(ref drawable)) = (self.draw_fn, self.drawable.state.as_ref()) {
            let bounds = self.layout.bounds(solver);
            let context = util::crop_context(context, crop_to);
            draw_fn(DrawArgs {
                state: drawable.as_ref(),
                bounds: bounds,
                parent_bounds: crop_to,
                glyph_cache: glyph_cache,
                context: context,
                graphics: graphics,
            });
        }
    }
    pub fn is_mouse_over(&self, solver: &mut Solver, mouse: Point) -> bool {
        let bounds = self.layout.bounds(solver);
        (self.mouse_over_fn)(mouse, bounds)
    }
    pub fn trigger_event(&mut self,
                         event: &(Event + 'static),
                         event_queue: &mut EventQueue,
                         solver: &mut Solver) {
        if let Some(event_handler) = self.event_handlers
            .iter_mut()
            .find(|event_handler| event_handler.event_id() == event.event_id()) {
            event_handler.handle_event(EventArgs {
                event: event,
                widget_id: self.id,
                state: &mut self.drawable,
                layout: &mut self.layout,
                event_queue: event_queue,
                solver: solver,
            });
        } else {
            // no event handler for id
            // println!("widget {:?} has no handler for {:?}", self.id, id);
        }
    }
}

pub struct DrawableEventHandler<T> {
    event_id: EventId,
    drawable_callback: Box<Fn(&mut T)>,
}
impl<T: 'static> DrawableEventHandler<T> {
    pub fn new<H: Fn(&mut T) + 'static>(event_id: EventId, drawable_callback: H) -> Self {
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
    fn handle_event(&mut self, mut event_args: EventArgs) {
        (self.drawable_callback)(event_args.state.state::<T>());
    }
}