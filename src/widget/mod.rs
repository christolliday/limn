#[macro_use]
pub mod style;
pub mod property;
pub mod drawable;

use std::any::{TypeId, Any};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::{RefCell};

use graphics::Context;
use graphics::types::Color;

use backend::gfx::G2d;
use backend::glyph::GlyphCache;

use event::{WidgetEventHandler, WidgetEventArgs, WidgetHandlerWrapper};
use layout::{LayoutManager, Layout, LayoutVars, LayoutRef};
use layout::container::{LayoutContainer, Frame};
use resources::{resources, WidgetId};
use util::{self, Point, Rect};
use event::Target;
use layout::UpdateLayout;

use self::property::{PropSet, Property};
use self::drawable::{Drawable, DrawableWrapper};
use self::style::Style;

pub struct WidgetBuilder {
    id: WidgetId,
    drawable: Option<DrawableWrapper>,
    props: PropSet,
    container: Option<Box<LayoutContainer>>,
    pub layout: Layout,
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
impl AsRef<WidgetBuilder> for WidgetBuilder {
    fn as_ref(&self) -> &WidgetBuilder {
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
impl LayoutRef for WidgetBuilder {
    fn layout_ref(&self) -> &LayoutVars {
        &self.as_ref().layout.vars
    }
}
impl LayoutRef for Widget {
    fn layout_ref(&self) -> &LayoutVars {
        &self.layout.vars
    }
}

#[macro_export]
macro_rules! widget_builder {
    ($builder_type:ty) => {
        impl AsMut<WidgetBuilder> for $builder_type {
            fn as_mut(&mut self) -> &mut WidgetBuilder {
                &mut self.widget
            }
        }
        impl AsRef<WidgetBuilder> for $builder_type {
            fn as_ref(&self) -> &WidgetBuilder {
                &self.widget
            }
        }
        impl LayoutRef for $builder_type {
            fn layout_ref(&self) -> &LayoutVars {
                &self.as_ref().layout.vars
            }
        }
        impl BuildWidget for $builder_type {
            fn build(self) -> WidgetBuilder {
                self.widget
            }
        }
    };
    ($builder_type:ty, build: $build_expr:expr) => {
        impl AsMut<WidgetBuilder> for $builder_type {
            fn as_mut(&mut self) -> &mut WidgetBuilder {
                &mut self.widget
            }
        }
        impl AsRef<WidgetBuilder> for $builder_type {
            fn as_ref(&self) -> &WidgetBuilder {
                &self.widget
            }
        }
        impl LayoutRef for $builder_type {
            fn layout_ref(&self) -> &LayoutVars {
                &self.as_ref().layout.vars
            }
        }
        impl BuildWidget for $builder_type {
            fn build(self) -> WidgetBuilder {
                $build_expr(self)
            }
        }
    };
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
    fn no_container(&mut self) -> &mut Self;
    fn set_container<C: LayoutContainer + 'static>(&mut self, container: C) -> &mut Self;
    fn set_padding(&mut self, padding: f64) -> &mut Self;
    fn layout(&mut self) -> &mut Layout;
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
        for child in &mut self.as_mut().children {
            child.props.insert(Property::Inactive);
        }
        self
    }
    fn add_child<C: BuildWidget>(&mut self, widget: C) -> &mut Self {
        self.as_mut().children.push(widget.build());
        self
    }
    fn no_container(&mut self) -> &mut Self {
        self.as_mut().container = None;
        self
    }
    fn set_container<C: LayoutContainer + 'static>(&mut self, container: C) -> &mut Self {
        self.as_mut().container = Some(Box::new(container));
        self
    }
    fn set_padding(&mut self, padding: f64) -> &mut Self {
        self.as_mut().container.as_mut().unwrap().set_padding(padding);
        self
    }
    fn layout(&mut self) -> &mut Layout {
        &mut self.as_mut().layout
    }
    fn id(&mut self) -> WidgetId {
        self.as_mut().id
    }
}
impl WidgetBuilder {
    pub fn new() -> Self {
        WidgetBuilder::new_widget(None)
    }
    pub fn new_named(name: &str) -> Self {
        WidgetBuilder::new_widget(Some(name.to_owned()))
    }
    fn new_widget(name: Option<String>) -> Self {
        let mut builder = WidgetBuilder {
            id: resources().widget_id(),
            drawable: None,
            props: PropSet::new(),
            container: Some(Box::new(Frame::new())),
            layout: Layout::new(None),
            handlers: HashMap::new(),
            debug_name: name,
            debug_color: None,
            children: Vec::new(),
        };
        builder.add_handler_fn(property::prop_change_handle);
        builder
    }

    pub fn build(mut self) -> (Vec<WidgetBuilder>, WidgetRef) {
        self.layout.name = self.debug_name.clone();
        let mut widget = Widget::new(self.id,
                                 self.drawable,
                                 self.props,
                                 self.layout,
                                 self.debug_name,
                                 self.debug_color);
        widget.apply_style();
        let widget = WidgetContainer {
            widget: widget,
            container: self.container,
            handlers: self.handlers,
        };
        (self.children, WidgetRef(Rc::new(RefCell::new(widget))))
    }
}

pub struct WidgetContainer {
    pub widget: Widget,
    pub container: Option<Box<LayoutContainer>>,
    pub handlers: HashMap<TypeId, Vec<WidgetHandlerWrapper>>,
}
impl WidgetContainer {
    pub fn trigger_event(&mut self,
                         type_id: TypeId,
                         event: &Box<Any + Send>,
                         solver: &mut LayoutManager)
                         -> bool {
        let mut handled = false;
        if let Some(handlers) = self.handlers.get_mut(&type_id) {
            for event_handler in handlers.iter_mut() {
                let event_args = WidgetEventArgs {
                    widget: &mut self.widget,
                    solver: solver,
                    handled: &mut handled,
                };
                event_handler.handle(event, event_args);
            }
        }
        handled
    }
}

#[derive(Clone)]
pub struct WidgetRef(pub Rc<RefCell<WidgetContainer>>);

use std::cell::RefMut;
impl WidgetRef {
    pub fn widget_container(&self) -> RefMut<WidgetContainer> {
        self.0.borrow_mut()
    }
    pub fn id(&self) -> WidgetId {
        self.0.borrow().widget.id
    }
    pub fn debug_name(&self) -> Option<String> {
        self.0.borrow().widget.debug_name.clone()
    }
    pub fn debug_color(&self) -> Option<Color> {
        self.0.borrow().widget.debug_color
    }
    pub fn has_updated(&self) -> bool {
        self.0.borrow().widget.has_updated
    }
    pub fn set_updated(&self, has_updated: bool) {
        self.0.borrow_mut().widget.has_updated = has_updated;
    }
    pub fn bounds(&self) -> Rect {
        self.0.borrow().widget.bounds
    }
}

pub struct Widget {
    pub id: WidgetId,
    pub drawable: Option<DrawableWrapper>,
    pub props: PropSet,
    pub has_updated: bool,
    pub layout: Layout,
    pub bounds: Rect,
    pub debug_name: Option<String>,
    pub debug_color: Option<Color>,
}

impl Widget {
    pub fn new(id: WidgetId,
               drawable: Option<DrawableWrapper>,
               props: PropSet,
               layout: Layout,
               debug_name: Option<String>,
               debug_color: Option<Color>)
               -> Self {
        let widget = Widget {
            id: id,
            drawable: drawable,
            props: props,
            layout: layout,
            has_updated: false,
            bounds: Rect::zero(),
            debug_name: debug_name,
            debug_color: debug_color,
        };
        widget
    }
    pub fn layout(&mut self) -> &mut Layout {
        &mut self.layout
    }
    pub fn draw(&mut self,
                crop_to: Rect,
                glyph_cache: &mut GlyphCache,
                context: Context,
                graphics: &mut G2d) {

        if let Some(drawable) = self.drawable.as_mut() {
            let bounds = self.bounds;
            let context = util::crop_context(context, crop_to);
            drawable.drawable.draw(bounds, crop_to, glyph_cache, context, graphics);
        }
    }

    pub fn is_mouse_over(&self, mouse: Point) -> bool {
        self.bounds.contains(&mouse)
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

    pub fn update_layout<F>(&mut self, f: F)
        where F: FnOnce(&mut Layout)
    {
        f(&mut self.layout);
        event!(Target::Ui, UpdateLayout(self.id));
    }

    pub fn apply_style(&mut self) {
        if let Some(ref mut drawable) = self.drawable {
            if drawable.apply_style(&self.props) {
                self.has_updated = true;
            }
        }
    }
}
