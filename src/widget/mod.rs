pub mod layout;
pub mod style;
pub mod property;
pub mod drawable;

use std::any::{TypeId, Any};
use std::marker::PhantomData;

use graphics::Context;
use graphics::types::Color;

use cassowary::Constraint;

use backend::gfx::G2d;
use backend::glyph::GlyphCache;

use event::Queue;
use ui::solver::LimnSolver;
use resources::{resources, WidgetId};
use util::{self, Point, Rectangle};

use self::property::{PropSet, Property, PropChangeHandler};
use self::layout::{LayoutBuilder, LayoutVars};
use self::drawable::{Drawable, DrawableWrapper};
use self::style::Style;

pub struct WidgetBuilder {
    pub id: WidgetId,
    pub drawable: Option<DrawableWrapper>,
    pub props: PropSet,
    pub layout: LayoutBuilder,
    pub controller: WidgetController,
    pub debug_name: Option<String>,
    pub debug_color: Option<Color>,
    pub children: Vec<WidgetBuilder>,
    pub contents_scroll: bool,
}

impl WidgetBuilder {
    pub fn new() -> Self {
        WidgetBuilder {
            id: resources().widget_id(),
            drawable: None,
            props: PropSet::new(),
            layout: LayoutBuilder::new(),
            controller: WidgetController::new(),
            debug_name: None,
            debug_color: None,
            children: Vec::new(),
            contents_scroll: false,
        }
    }
    pub fn set_drawable<T: Drawable + 'static>(&mut self, drawable: T) -> &mut Self {
        self.drawable = Some(DrawableWrapper::new(drawable));
        self
    }
    pub fn set_drawable_with_style<T: Drawable + 'static, S: Style<T> + 'static>(&mut self, drawable: T, style: S) -> &mut Self {
        self.drawable = Some(DrawableWrapper::new_with_style(drawable, style));
        self
    }
    pub fn add_handler<E: 'static, T: EventHandler<E> + 'static>(&mut self, handler: T) -> &mut Self {
        self.controller.add_handler(handler);
        self
    }
    pub fn set_debug_name(&mut self, name: &str) -> &mut Self {
        self.debug_name = Some(name.to_owned());
        self
    }
    pub fn set_debug_color(&mut self, color: Color) -> &mut Self {
        self.debug_color = Some(color);
        self
    }
    pub fn set_inactive(&mut self) -> &mut Self {
        self.props.insert(Property::Inactive);
        self
    }
    pub fn add_child(&mut self, mut widget: WidgetBuilder) -> &mut Self {
        if self.contents_scroll {
            widget.layout.scroll_inside(&self.layout.vars);
        } else {
            widget.layout.bound_by(&self.layout.vars, None);
        }
        self.children.push(widget);
        self
    }

    pub fn build(self) -> (Vec<WidgetBuilder>, Vec<Constraint>, WidgetContainer) {

        let widget = Widget::new(self.id,
                                 self.drawable,
                                 self.props,
                                 self.layout.vars,
                                 self.debug_name,
                                 self.debug_color);
        (self.children,
         self.layout.constraints,
         WidgetContainer {
             widget: widget,
             controller: self.controller,
         })
    }
}

pub struct WidgetController {
    handlers: Vec<HandlerWrapper>,
}
impl WidgetController {
    pub fn new() -> Self {
        let mut controller = WidgetController {
            handlers: Vec::new(),
        };
        controller.add_handler(PropChangeHandler);
        controller
    }
    pub fn add_handler<H: EventHandler<E> + 'static, E: 'static>(&mut self, handler: H) {
        self.handlers.push(HandlerWrapper::new(handler));
    }
}
struct HandlerWrapper {
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

// allows event handlers to communicate with event dispatcher
pub struct EventState {
    pub handled: bool,
}
pub struct EventArgs<'a> {
    pub widget: &'a mut Widget,
    pub queue: &'a mut Queue,
    pub solver: &'a mut LimnSolver,
    pub event_state: &'a mut EventState,
}

pub trait EventHandler<T> {
    fn handle(&mut self, event: &T, args: EventArgs);
}

pub struct CallbackHandler<F, E>
    where F: Fn(&E, &mut EventArgs)
{
    callback: F,
    phantom: PhantomData<E>,
}
impl<F, E> CallbackHandler<F, E>
    where F: Fn(&E, &mut EventArgs) {
    pub fn new(callback: F) -> Self {
        CallbackHandler {
            callback: callback,
            phantom: PhantomData,
        }
    }
}
impl<F, E> EventHandler<E> for CallbackHandler<F, E>
    where F: Fn(&E, &mut EventArgs) {
    fn handle(&mut self, event: &E, mut args: EventArgs) {
        (self.callback)(event, &mut args);
    }
}

pub struct WidgetContainer {
    pub widget: Widget,
    pub controller: WidgetController,
}
impl WidgetContainer {
    pub fn trigger_event(&mut self,
                         type_id: TypeId,
                         event: &Box<Any + Send>,
                         queue: &mut Queue,
                         solver: &mut LimnSolver)
                         -> bool {

        let mut event_state = EventState { handled: false };
        for ref mut event_handler in self.controller.handlers.iter_mut() {
            let event_handler: &mut HandlerWrapper = event_handler;
            if event_handler.handles(type_id) {
                let event_args = EventArgs {
                    widget: &mut self.widget,
                    queue: queue,
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
    pub fn drawable<T: Drawable>(&self) -> Option<&T> {
        if let Some(ref drawable) = self.drawable {
            drawable.drawable.as_ref().downcast_ref::<T>()
        } else {
            None
        }
    }

    pub fn update<F, T: Drawable + 'static>(&mut self, f: F)
        where F: FnOnce(&mut T)
    {
        if let Some(ref mut drawable) = self.drawable {
            self.has_updated = true;
            let state = drawable.drawable.as_mut().downcast_mut::<T>().expect("Called update on widget with wrong drawable type");
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
