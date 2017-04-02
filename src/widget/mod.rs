pub mod style;
pub mod property;
pub mod drawable;

use std::any::{TypeId, Any};
use std::collections::HashMap;

use graphics::Context;
use graphics::types::Color;

use cassowary::Constraint;

use backend::gfx::G2d;
use backend::glyph::GlyphCache;

use event::{Queue, WidgetEventHandler, WidgetEventArgs, WidgetHandlerWrapper};
use layout::solver::LimnSolver;
use layout::{LayoutBuilder, LayoutVars};
use resources::{resources, WidgetId};
use util::{self, Point, Rectangle};

use self::property::{PropSet, Property};
use self::drawable::{Drawable, DrawableWrapper};
use self::style::Style;

pub struct WidgetBuilder {
    id: WidgetId,
    drawable: Option<DrawableWrapper>,
    props: PropSet,
    layout: LayoutBuilder,
    pub bound_children: bool,
    handlers: HashMap<TypeId, Vec<WidgetHandlerWrapper>>,
    debug_name: Option<String>,
    debug_color: Option<Color>,
    children: Vec<WidgetBuilder>,
}

impl AsMut<WidgetBuilder> for WidgetBuilder {
    fn as_mut(&mut self) -> &mut WidgetBuilder {
        self
    }
}
pub trait BuildWidget: AsMut<WidgetBuilder> {
    fn build(self) -> WidgetBuilder;
}
impl BuildWidget for WidgetBuilder {
    fn build(self) -> WidgetBuilder {
        self
    }
}

pub trait WidgetBuilderCore {
    fn set_drawable<T: Drawable + 'static>(&mut self, drawable: T) -> &mut Self;
    fn set_drawable_with_style<T: Drawable + 'static, S: Style<T> + 'static>(&mut self, drawable: T, style: S) -> &mut Self;
    fn add_handler<E: 'static, T: WidgetEventHandler<E> + 'static>(&mut self, handler: T) -> &mut Self;
    fn add_handler_fn<E: 'static, T: Fn(&E, WidgetEventArgs) + 'static>(&mut self, handler: T) -> &mut Self;
    fn set_debug_name(&mut self, name: &str) -> &mut Self;
    fn set_debug_color(&mut self, color: Color) -> &mut Self;
    fn set_inactive(&mut self) -> &mut Self;
    fn add_child<C: BuildWidget>(&mut self, widget: C) -> &mut Self;
    fn layout(&mut self) -> &mut LayoutBuilder;
    fn id(&mut self) -> WidgetId;
}

impl<B> WidgetBuilderCore for B where B: AsMut<WidgetBuilder> {
    fn set_drawable<T: Drawable + 'static>(&mut self, drawable: T) -> &mut Self {
        self.as_mut().drawable = Some(DrawableWrapper::new(drawable));
        self
    }
    fn set_drawable_with_style<T: Drawable + 'static, S: Style<T> + 'static>(&mut self, drawable: T, style: S) -> &mut Self {
        self.as_mut().drawable = Some(DrawableWrapper::new_with_style(drawable, style));
        self
    }
    fn add_handler<E: 'static, T: WidgetEventHandler<E> + 'static>(&mut self, handler: T) -> &mut Self {
        self.as_mut().handlers.entry(TypeId::of::<E>()).or_insert(Vec::new())
            .push(WidgetHandlerWrapper::new(handler));
        self
    }
    fn add_handler_fn<E: 'static, T: Fn(&E, WidgetEventArgs) + 'static>(&mut self, handler: T) -> &mut Self {
        self.as_mut().handlers.entry(TypeId::of::<E>()).or_insert(Vec::new())
            .push(WidgetHandlerWrapper::new_from_fn(handler));
        self
    }
    fn set_debug_name(&mut self, name: &str) -> &mut Self {
        self.as_mut().debug_name = Some(name.to_owned());
        self
    }
    fn set_debug_color(&mut self, color: Color) -> &mut Self {
        self.as_mut().debug_color = Some(color);
        self
    }
    fn set_inactive(&mut self) -> &mut Self {
        self.as_mut().props.insert(Property::Inactive);
        self
    }
    fn add_child<C: BuildWidget>(&mut self, widget: C) -> &mut Self {
        self.as_mut().children.push(widget.build());
        self
    }
    fn layout(&mut self) -> &mut LayoutBuilder {
        &mut self.as_mut().layout
    }
    fn id(&mut self) -> WidgetId {
        self.as_mut().id
    }
}
impl WidgetBuilder {
    pub fn new() -> Self {
        let mut builder = WidgetBuilder {
            id: resources().widget_id(),
            drawable: None,
            props: PropSet::new(),
            layout: LayoutBuilder::new(),
            bound_children: true,
            handlers: HashMap::new(),
            debug_name: None,
            debug_color: None,
            children: Vec::new(),
        };
        builder.add_handler_fn(property::prop_change_handle);
        builder
    }

    pub fn build(self) -> (Vec<WidgetBuilder>, Vec<Constraint>, WidgetContainer) {
        let widget = Widget::new(self.id,
                                 self.drawable,
                                 self.props,
                                 self.layout.vars,
                                 self.bound_children,
                                 self.debug_name,
                                 self.debug_color);
        (self.children,
         self.layout.constraints,
         WidgetContainer {
             widget: widget,
             handlers: self.handlers,
         })
    }
}

pub struct WidgetContainer {
    pub widget: Widget,
    pub handlers: HashMap<TypeId, Vec<WidgetHandlerWrapper>>,
}
impl WidgetContainer {
    pub fn trigger_event(&mut self,
                         type_id: TypeId,
                         event: &Box<Any + Send>,
                         queue: &mut Queue,
                         solver: &mut LimnSolver)
                         -> bool {
        let mut handled = false;
        if let Some(handlers) = self.handlers.get_mut(&type_id) {
            for event_handler in handlers.iter_mut() {
                let event_args = WidgetEventArgs {
                    widget: &mut self.widget,
                    queue: queue,
                    solver: solver,
                    handled: &mut handled,
                };
                event_handler.handle(event, event_args);
            }
        }
        handled
    }
}
pub struct Widget {
    pub id: WidgetId,
    pub drawable: Option<DrawableWrapper>,
    pub props: PropSet,
    pub has_updated: bool,
    pub layout: LayoutVars,
    pub bound_children: bool,
    pub debug_name: Option<String>,
    pub debug_color: Option<Color>,
}

impl Widget {
    pub fn new(id: WidgetId,
               drawable: Option<DrawableWrapper>,
               props: PropSet,
               layout: LayoutVars,
               bound_children: bool,
               debug_name: Option<String>,
               debug_color: Option<Color>)
               -> Self {
        let mut widget = Widget {
            id: id,
            drawable: drawable,
            props: props,
            has_updated: false,
            layout: layout,
            bound_children: bound_children,
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

    pub fn update_layout<F>(&mut self, f: F, solver: &mut LimnSolver)
        where F: FnOnce(&mut LayoutBuilder)
    {
        let mut layout = LayoutBuilder::from(self.layout.clone());
        f(&mut layout);
        solver.add_constraints(layout.constraints);
    }
}
