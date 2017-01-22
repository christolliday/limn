pub mod layout;
pub mod primitives;
pub mod text;
pub mod image;
pub mod button;
pub mod scroll;
pub mod builder;

use backend::gfx::G2d;
use backend::glyph::GlyphCache;
use graphics::Context;
use graphics::types::Color;

use event::Event;
use eventbus::{EventBus, EventAddress};
use input::EventId;
use super::util::*;
use super::util;
use super::resources::Id;

use ui::Ui;
use ui::EventQueue;
use super::ui::Resources;
use self::layout::WidgetLayout;

use cassowary::Solver;
use cassowary::strength::*;

use std::any::Any;

pub struct DrawArgs<'a, 'b: 'a> {
    pub state: &'a Any,
    pub bounds: Rectangle,
    pub parent_bounds: Rectangle,
    pub resources: &'a Resources,
    pub glyph_cache: &'a mut GlyphCache,
    pub context: Context,
    pub graphics: &'a mut G2d<'b>,
}

pub struct EventArgs<'a> {
    pub event: &'a Event,
    pub widget_id: Id,
    pub state: Option<&'a mut Any>,
    pub layout: &'a mut WidgetLayout,
    pub parent_layout: &'a WidgetLayout,
    pub event_queue: &'a mut EventQueue,
    pub solver: &'a mut Solver,
}

pub trait EventHandler {
    fn event_id(&self) -> EventId;
    fn handle_event(&mut self, event_args: EventArgs);
}

pub struct Widget {
    pub id: Id,
    pub draw_fn: Option<fn(DrawArgs)>,
    pub drawable: Option<Box<Any>>,
    pub mouse_over_fn: fn(Point, Rectangle) -> bool,
    pub layout: WidgetLayout,
    pub event_handlers: Vec<Box<EventHandler>>,
    pub debug_color: Color,
}

use input::{Input, Motion};
impl Widget {
    pub fn new(id: Id) -> Self {
        Widget {
            id: id,
            draw_fn: None,
            drawable: None,
            mouse_over_fn: point_inside_rect,
            layout: WidgetLayout::new(),
            event_handlers: Vec::new(),
            debug_color: [0.0, 1.0, 0.0, 1.0],
        }
    }
    pub fn set_drawable(&mut self, draw_fn: fn(DrawArgs), drawable: Box<Any>) {
        self.draw_fn = Some(draw_fn);
        self.drawable = Some(drawable);
    }
    pub fn set_mouse_over_fn(&mut self, mouse_over_fn: fn(Point, Rectangle) -> bool) {
        self.mouse_over_fn = mouse_over_fn;
    }
    pub fn debug_color(&mut self, color: Color) {
        self.debug_color = color;
    }
    pub fn draw(&self, crop_to: Rectangle, resources: &Resources, solver: &mut Solver, glyph_cache: &mut GlyphCache, context: Context, graphics: &mut G2d) {
        if let (Some(draw_fn), Some(ref drawable)) = (self.draw_fn, self.drawable.as_ref()) {
            let bounds = self.layout.bounds(solver);
            let context = util::crop_context(context, crop_to);
            draw_fn(DrawArgs {
                state: drawable.as_ref(),
                bounds: bounds,
                parent_bounds: crop_to,
                resources: resources,
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
    pub fn trigger_event(&mut self, id: EventId, event: &Event, event_queue: &mut EventQueue, parent_layout: &WidgetLayout, solver: &mut Solver) {
        if let Some(event_handler) = self.event_handlers.iter_mut().find(|event_handler| event_handler.event_id() == id) {
            let drawable = self.drawable.as_mut().map(|draw| draw.as_mut());
            event_handler.handle_event(EventArgs {
                event: event,
                widget_id: self.id,
                state: drawable,
                layout: &mut self.layout,
                parent_layout: parent_layout,
                event_queue: event_queue,
                solver: solver,
            });
        } else {
            // no event handler for id
            println!("widget {:?} has no handler for {:?}", self.id, id);
        }
    }
}



pub struct DrawableEventHandler<T>
{
    event_id: EventId,
    drawable_callback: Box<Fn(&mut T)>
}
impl<T: 'static> DrawableEventHandler<T>
{
    pub fn new<H: Fn(&mut T) + 'static>(event_id: EventId, drawable_callback: H) -> Self {
        DrawableEventHandler {
            event_id: event_id,
            drawable_callback: Box::new(drawable_callback),
        }
    }
}
impl<T: 'static> EventHandler for DrawableEventHandler<T>
{
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn handle_event(&mut self, event_args: EventArgs) {
        let EventArgs { state, .. } = event_args;
        let state = state.unwrap();
        let state = state.downcast_mut::<T>().unwrap();
        (self.drawable_callback)(state);
    }
}