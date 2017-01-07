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

use event::{Event, LimnEvent};
use input::EventId;
use super::util::*;
use super::util;

use super::ui::Resources;
use self::layout::WidgetLayout;

use cassowary::Solver;
use cassowary::strength::*;

use std::any::Any;

pub struct DrawArgs<'a, 'b: 'a> {
    state: &'a Any,
    bounds: Rectangle,
    parent_bounds: Rectangle,
    resources: &'a Resources,
    glyph_cache: &'a mut GlyphCache,
    context: Context,
    graphics: &'a mut G2d<'b>,
}

pub struct EventArgs<'a> {
    event: &'a LimnEvent,
    state: Option<&'a mut Any>,
    layout: &'a mut WidgetLayout,
    parent_layout: &'a WidgetLayout,
    solver: &'a mut Solver,
}

pub trait EventHandler {
    fn event_id(&self) -> EventId;
    fn handle_event(&mut self, event_args: EventArgs) -> Option<Box<LimnEvent>>;
}

pub struct Widget {
    pub draw_fn: Option<fn(DrawArgs)>,
    pub drawable: Option<Box<Any>>,
    pub mouse_over_fn: fn(Point, Rectangle) -> bool,
    pub layout: WidgetLayout,
    pub event_handlers: Vec<Box<EventHandler>>,
    pub debug_color: Color,
}

use input::{Input, Motion};
impl Widget {
    pub fn new() -> Self {
        Widget {
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
    pub fn trigger_event(&mut self, id: EventId, event: &LimnEvent, parent_layout: &WidgetLayout, solver: &mut Solver) -> Option<Box<LimnEvent>> {
        let event_handler = self.event_handlers.iter_mut().find(|event_handler| event_handler.event_id() == id).unwrap();

        let drawable = self.drawable.as_mut().map(|draw| draw.as_mut());
        event_handler.handle_event(EventArgs {
            event: event,
            state: drawable,
            layout: &mut self.layout,
            parent_layout: parent_layout,
            solver: solver,
        })
    }
}



pub struct DrawableEventHandler<D> {
    event_id: EventId,
    drawable_callback: fn(&mut D)
}
impl<D> DrawableEventHandler<D> {
    pub fn new(event_id: EventId, drawable_callback: fn(&mut D)) -> Self {
        DrawableEventHandler {
            event_id: event_id,
            drawable_callback: drawable_callback,
        }
    }
}
impl<D: 'static> EventHandler for DrawableEventHandler<D> {
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn handle_event(&mut self, event_args: EventArgs) -> Option<Box<LimnEvent>> {
        let EventArgs { state, .. } = event_args;
        let state = state.unwrap();
        let state = state.downcast_mut::<D>().unwrap();
        (self.drawable_callback)(state);
        None
    }
}