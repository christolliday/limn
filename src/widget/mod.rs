pub mod layout;
pub mod builder;
pub mod style;

use std::any::Any;
use std::collections::BTreeSet;

use graphics::Context;
use graphics::types::Color;
use cassowary::Solver;

use backend::gfx::G2d;
use backend::glyph::GlyphCache;

#[macro_use]
use event::{self, EventAddress, EventId, EventQueue, WIDGET_PROPS_CHANGED, WIDGET_CHANGE_PROP};
use resources::Id;
use util::{self, Point, Rectangle};

use self::builder::WidgetBuilder;
use self::layout::WidgetLayout;
use self::style::DrawableStyle;

#[derive(Hash, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub enum Property {
    Hover,
    Activated,
    Selected,
    Pressed,
}
pub type PropSet = BTreeSet<Property>;

pub struct DrawArgs<'a, 'b: 'a> {
    pub state: &'a Any,
    pub props: &'a PropSet,
    pub bounds: Rectangle,
    pub parent_bounds: Rectangle,
    pub glyph_cache: &'a mut GlyphCache,
    pub context: Context,
    pub graphics: &'a mut G2d<'b>,
}

pub struct EventArgs<'a> {
    pub data: &'a (Any + 'static),
    pub widget_id: Id,
    pub drawable: &'a mut Option<Drawable>,
    pub props: &'a mut PropSet,
    pub layout: &'a mut WidgetLayout,
    pub event_queue: &'a mut EventQueue,
    pub solver: &'a mut Solver,
}

pub struct StyleArgs<'a> {
    pub state: &'a mut Any,
    pub style: &'a Any,
    pub props: &'a PropSet,
}

pub trait EventHandler {
    fn event_id(&self) -> EventId;
    fn handle_event(&mut self, args: EventArgs);
}

pub struct Drawable {
    pub state: WidgetState,
    pub draw_fn: fn(DrawArgs),
    pub mouse_over_fn: Option<fn(Point, Rectangle) -> bool>,
    pub style: Option<WidgetStyle>,
}

pub struct WidgetStyle {
    pub style: Box<Any>,
    pub style_fn: fn(StyleArgs),
}

pub struct WidgetState {
    state: Box<Any>,
    pub has_updated: bool,
}
impl WidgetState {
    pub fn new(state: Box<Any>) -> Self {
        WidgetState { state: state, has_updated: false }
    }
    pub fn update<F, T: 'static>(&mut self, f: F) where F: FnOnce(&mut T) {
        self.has_updated = true;
        let state = self.state.as_mut().downcast_mut::<T>().unwrap();
        f(state);
    }
    pub fn state<T: 'static>(&self) -> &T {
        self.state.as_ref().downcast_ref::<T>().unwrap()
    }
}

pub struct Widget {
    pub id: Id,
    pub drawable: Option<Drawable>,
    pub layout: WidgetLayout,
    pub props: PropSet,
    pub event_handlers: Vec<Box<EventHandler>>,
    pub debug_name: Option<String>,
    pub debug_color: Option<Color>,
}

fn apply_style(state: &mut WidgetState, style: &Box<Any>, style_fn: fn(StyleArgs), props: &PropSet) {
    style_fn(StyleArgs {
        state: state.state.as_mut(),
        style: style.as_ref(),
        props: props,
    });
    state.has_updated = true;
}
impl Widget {
    pub fn new(id: Id,
               mut drawable: Option<Drawable>,
               layout: WidgetLayout,
               event_handlers: Vec<Box<EventHandler>>,
               debug_name: Option<String>,
               debug_color: Option<Color>,
               ) -> Self {

        let props = BTreeSet::new();
        if let Some(ref mut drawable) = drawable {
            if let Some(ref style) = drawable.style {
                apply_style(&mut drawable.state, &style.style, style.style_fn, &props);
            }
        }
        Widget {
            id: id,
            drawable: drawable,
            layout: layout,
            props: props,
            event_handlers: event_handlers,
            debug_name: debug_name,
            debug_color: debug_color,
        }
    }
    pub fn draw(&mut self,
                crop_to: Rectangle,
                solver: &mut Solver,
                glyph_cache: &mut GlyphCache,
                context: Context,
                graphics: &mut G2d) {

        if let Some(drawable) = self.drawable.as_mut() {
            let bounds = self.layout.bounds(solver);
            let context = util::crop_context(context, crop_to);
            (drawable.draw_fn)(DrawArgs {
                state: drawable.state.state.as_ref(),
                props: &self.props,
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
        if let Some(ref drawable) = self.drawable {
            if let Some(mouse_over_fn) = drawable.mouse_over_fn {
                return mouse_over_fn(mouse, bounds)
            }
        }
        util::point_inside_rect(mouse, bounds)
    }
    pub fn trigger_event(&mut self,
                         event_id: EventId,
                         data: &(Any + 'static),
                         event_queue: &mut EventQueue,
                         solver: &mut Solver) {

        for ref mut event_handler in self.event_handlers.iter_mut() {
            if event_handler.event_id() == event_id {
                event_handler.handle_event(EventArgs {
                    data: data,
                    widget_id: self.id,
                    drawable: &mut self.drawable,
                    layout: &mut self.layout,
                    props: &mut self.props,
                    event_queue: event_queue,
                    solver: solver,
                });
            }
        }
    }
}

pub struct PropsChangeEventHandler {}
impl EventHandler for PropsChangeEventHandler {
    fn event_id(&self) -> EventId {
        WIDGET_CHANGE_PROP
    }
    fn handle_event(&mut self, args: EventArgs) {
        let &(ref prop, add) = args.data.downcast_ref::<(Property, bool)>().unwrap();
        if add {
            args.props.insert(prop.clone());
        } else {
            args.props.remove(prop);
        }
        if let Some(drawable) = args.drawable.as_mut() {
            if let Some(ref style) = drawable.style {
                apply_style(&mut drawable.state, &style.style, style.style_fn, args.props);
                args.event_queue.push(EventAddress::Widget(args.widget_id), WIDGET_PROPS_CHANGED, Box::new(()));
            }
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
            drawable.state.update(|state: &mut T|
                (self.drawable_callback)(state)
            );
        }
    }
}