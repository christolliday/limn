pub mod layout;
pub mod builder;
pub mod style;
pub mod property;
pub mod drawable;

use std::any::Any;

use graphics::Context;
use graphics::types::Color;

use backend::gfx::G2d;
use backend::glyph::GlyphCache;

use event::{EventId, EventQueue};
use resources::WidgetId;
use layout::LimnSolver;
use util::{self, Point, Rectangle};
use ui::InputState;

use self::property::PropSet;
use self::layout::LayoutVars;
use self::drawable::Drawable;

pub struct EventArgs<'a> {
    pub data: &'a (Any + 'static),
    pub widget_id: WidgetId,
    pub drawable: &'a mut Option<Drawable>,
    pub layout: &'a mut LayoutVars,
    pub event_queue: &'a mut EventQueue,
    pub solver: &'a mut LimnSolver,
    pub input_state: &'a InputState,
    pub event_state: &'a mut EventState,
}

// allows event handlers to communicate with event dispatcher
pub struct EventState {
    pub handled: bool,
}

pub trait EventHandler {
    fn event_id(&self) -> EventId;
    fn handle_event(&mut self, args: EventArgs);
}

pub struct Widget {
    pub id: WidgetId,
    pub drawable: Option<Drawable>,
    pub layout: LayoutVars,
    pub event_handlers: Vec<Box<EventHandler>>,
    pub debug_name: Option<String>,
    pub debug_color: Option<Color>,
}

impl Widget {
    pub fn new(id: WidgetId,
               drawable: Option<Drawable>,
               layout: LayoutVars,
               event_handlers: Vec<Box<EventHandler>>,
               debug_name: Option<String>,
               debug_color: Option<Color>)
               -> Self
    {
        Widget {
            id: id,
            drawable: drawable,
            layout: layout,
            event_handlers: event_handlers,
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
    pub fn trigger_event(&mut self,
                         event_id: EventId,
                         data: &(Any + 'static),
                         event_queue: &mut EventQueue,
                         solver: &mut LimnSolver,
                         input_state: &InputState) -> bool {

        let mut event_state = EventState { handled: false };
        for ref mut event_handler in self.event_handlers.iter_mut() {
            if event_handler.event_id() == event_id {
                event_handler.handle_event(EventArgs {
                    data: data,
                    widget_id: self.id,
                    drawable: &mut self.drawable,
                    layout: &mut self.layout,
                    event_queue: event_queue,
                    solver: solver,
                    input_state: input_state,
                    event_state: &mut event_state,
                });
            }
        }
        event_state.handled
    }
}
