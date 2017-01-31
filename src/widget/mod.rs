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
use event::{self, EventAddress, EventId, Event, Signal, EventQueue};
use resources::Id;
use util::{self, Point, Rectangle};

use self::builder::WidgetBuilder;
use self::layout::WidgetLayout;
use self::style::DrawableStyle;

#[derive(Hash, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub enum WidgetProperty {
    Hover,
    Activated,
    Selected,
    Pressed,
}

pub struct ChangePropEvent {
    val: (WidgetProperty, bool),
}
impl ChangePropEvent {
    pub fn new(prop: WidgetProperty, add: bool) -> Self {
        ChangePropEvent { val: (prop, add) }
    }
}
impl Event for ChangePropEvent {
    fn event_id(&self) -> EventId {
        event::WIDGET_CHANGE_PROP
    }
    fn event_data(&self) -> Option<&Any> {
        Some(&self.val)
    }
}

pub struct PropsChangeEventHandler {}
impl EventHandler for PropsChangeEventHandler {
    fn event_id(&self) -> EventId {
        event::WIDGET_CHANGE_PROP
    }
    fn handle_event(&mut self, args: EventArgs) {
        let &(ref prop, add) = args.event.data::<(WidgetProperty, bool)>();
        if add {
            args.props.insert(prop.clone());
        } else {
            args.props.remove(prop);
        }
        args.event_queue.push(EventAddress::Widget(args.widget_id), Box::new(Signal::new(event::WIDGET_PROPS_CHANGED)));
    }
}

pub struct WidgetNotifyEvent {
    event_id: EventId,
    widget_id: Id,
}
impl WidgetNotifyEvent {
    pub fn new(event_id: EventId, widget_id: Id) -> Self {
        WidgetNotifyEvent {
            event_id: event_id,
            widget_id: widget_id,
        }
    }
}
impl Event for WidgetNotifyEvent {
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn event_data(&self) -> Option<&Any> {
        Some(&self.widget_id)
    }
}

pub struct DrawArgs<'a, 'b: 'a> {
    pub state: &'a Any,
    pub props: &'a BTreeSet<WidgetProperty>,
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
    pub props: &'a mut BTreeSet<WidgetProperty>,
    pub layout: &'a mut WidgetLayout,
    pub event_queue: &'a mut EventQueue,
    pub solver: &'a mut Solver,
}

pub struct StyleArgs<'a> {
    pub state: &'a mut Any,
    pub style: &'a Any,
    pub props: &'a BTreeSet<WidgetProperty>,
}

pub trait EventHandler {
    fn event_id(&self) -> EventId;
    fn handle_event(&mut self, mut event_args: EventArgs);
}

pub struct WidgetState {
    state: Option<Box<Any>>,
    pub has_updated: bool,
}
impl WidgetState {
    pub fn new() -> Self {
        WidgetState { state: None, has_updated: false }
    }
    pub fn new_state(state: Box<Any>) -> Self {
        WidgetState { state: Some(state), has_updated: false }
    }
    pub fn update<F, T: 'static>(&mut self, f: F) where F: FnOnce(&mut T) {
        self.has_updated = true;
        let state = self.state.as_mut().map(|state| state.as_mut()).unwrap().downcast_mut::<T>().unwrap();
        f(state);
    }
    pub fn state<T: 'static>(&self) -> &T {
        self.state.as_ref().map(|draw| draw.as_ref()).unwrap().downcast_ref::<T>().unwrap()
    }
}

pub struct Widget {
    pub id: Id,
    pub draw_fn: Option<fn(DrawArgs)>,
    pub drawable: WidgetState,
    pub style: Option<Box<Any>>,
    pub style_fn: Option<fn(StyleArgs)>,
    pub mouse_over_fn: fn(Point, Rectangle) -> bool,
    pub layout: WidgetLayout,
    pub props: BTreeSet<WidgetProperty>,
    pub event_handlers: Vec<Box<EventHandler>>,
    pub debug_name: Option<String>,
    pub debug_color: Option<Color>,
}

impl Widget {
    pub fn new(id: Id,
               draw_fn: Option<fn(DrawArgs)>,
               drawable: WidgetState,
               style: Option<Box<Any>>,
               style_fn: Option<fn(StyleArgs)>,
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
            style: style,
            style_fn: style_fn,
            mouse_over_fn: mouse_over_fn,
            layout: layout,
            props: BTreeSet::new(),
            event_handlers: event_handlers,
            debug_name: debug_name,
            debug_color: debug_color,
        }
    }
    pub fn apply_style(&mut self) {
        if let (Some(ref mut drawable), Some(style), Some(style_fn)) = (self.drawable.state.as_mut(), self.style.as_ref(), self.style_fn) {
            style_fn(StyleArgs {
                state: drawable.as_mut(),
                style: style.as_ref(),
                props: &self.props,
            });
        }
    }
    pub fn draw(&mut self,
                crop_to: Rectangle,
                solver: &mut Solver,
                glyph_cache: &mut GlyphCache,
                context: Context,
                graphics: &mut G2d) {

        self.apply_style();

        if let (Some(draw_fn), Some(ref mut drawable)) = (self.draw_fn, self.drawable.state.as_mut()) {
            let bounds = self.layout.bounds(solver);
            let context = util::crop_context(context, crop_to);
            draw_fn(DrawArgs {
                state: drawable.as_ref(),
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
        (self.mouse_over_fn)(mouse, bounds)
    }
    pub fn trigger_event(&mut self,
                         event: &(Event + 'static),
                         event_queue: &mut EventQueue,
                         solver: &mut Solver) {

        for ref mut event_handler in self.event_handlers.iter_mut() {
            if event_handler.event_id() == event.event_id() {
                event_handler.handle_event(EventArgs {
                    event: event,
                    widget_id: self.id,
                    state: &mut self.drawable,
                    layout: &mut self.layout,
                    props: &mut self.props,
                    event_queue: event_queue,
                    solver: solver,
                });
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
    fn handle_event(&mut self, mut event_args: EventArgs) {
        event_args.state.update(|state: &mut T|
            (self.drawable_callback)(state)
        );
    }
}