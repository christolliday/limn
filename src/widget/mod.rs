#[macro_use]
pub mod style;
pub mod property;
pub mod drawable;

use std::any::{TypeId, Any};
use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::cell::{RefCell, Ref, RefMut};
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};

use graphics::Context;
use graphics::types::Color;

use backend::gfx::G2d;
use backend::glyph::GlyphCache;

use event::{WidgetEventHandler, WidgetEventArgs, WidgetHandlerWrapper};
use layout::{Layout, LayoutVars, LayoutRef};
use layout::container::{LayoutContainer, Frame};
use resources::{resources, WidgetId};
use util::{self, Point, Rect};
use event::Target;
use layout::UpdateLayout;

use self::property::{PropSet, Property};
use self::drawable::{Drawable, DrawableWrapper};
use self::style::Style;

pub struct WidgetBuilder {
    children: Vec<WidgetBuilder>,
    pub widget: WidgetRef,
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
    fn layout_ref(&self) -> LayoutVars {
        self.as_ref().widget.widget_mut().layout.vars.clone()
    }
}
impl LayoutRef for WidgetRef {
    fn layout_ref(&self) -> LayoutVars {
        self.widget_mut().layout.vars.clone()
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
            fn layout_ref(&self) -> LayoutVars {
                self.as_ref().widget.widget_mut().layout.vars.clone()
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
            fn layout_ref(&self) -> LayoutVars {
                self.as_ref().widget.widget_mut().layout.vars.clone()
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
    fn add_handler_wrapper(&mut self, type_id: TypeId, handler: WidgetHandlerWrapper) -> &mut Self;
    fn set_debug_name(&mut self, name: &str) -> &mut Self;
    fn set_debug_color(&mut self, color: Color) -> &mut Self;
    fn set_inactive(&mut self) -> &mut Self;
    fn add_child<C: BuildWidget>(&mut self, widget: C) -> &mut Self;
    fn no_container(&mut self) -> &mut Self;
    fn set_container<C: LayoutContainer + 'static>(&mut self, container: C) -> &mut Self;
    fn set_padding(&mut self, padding: f64) -> &mut Self;
    fn layout(&mut self) -> LayoutGuard;
    fn id(&mut self) -> WidgetId;
}

impl<B> WidgetBuilderCore for B where B: AsMut<WidgetBuilder> {
    fn set_drawable<T: Drawable + 'static>(&mut self, drawable: T) -> &mut Self {
        self.as_mut().widget.widget_mut().drawable = Some(DrawableWrapper::new(drawable));
        self
    }
    fn set_drawable_with_style<T: Drawable + 'static, S: Style<T> + 'static>(&mut self, drawable: T, style: S) -> &mut Self {
        self.as_mut().widget.widget_mut().drawable = Some(DrawableWrapper::new_with_style(drawable, style));
        self
    }
    fn add_handler<E: 'static, T: WidgetEventHandler<E> + 'static>(&mut self, handler: T) -> &mut Self {
        self.add_handler_wrapper(TypeId::of::<E>(), WidgetHandlerWrapper::new(handler))
    }
    fn add_handler_fn<E: 'static, T: Fn(&E, WidgetEventArgs) + 'static>(&mut self, handler: T) -> &mut Self {
        self.add_handler_wrapper(TypeId::of::<E>(), WidgetHandlerWrapper::new_from_fn(handler))
    }
    fn add_handler_wrapper(&mut self, type_id: TypeId, handler: WidgetHandlerWrapper) -> &mut Self {
        self.as_mut().widget.widget_mut().handlers.entry(type_id).or_insert(Vec::new())
            .push(Rc::new(RefCell::new(handler)));
        self
    }
    fn set_debug_name(&mut self, name: &str) -> &mut Self {
        self.as_mut().widget.widget_mut().debug_name = Some(name.to_owned());
        self.as_mut().widget.widget_mut().layout.name = Some(name.to_owned());
        self
    }
    fn set_debug_color(&mut self, color: Color) -> &mut Self {
        self.as_mut().widget.widget_mut().debug_color = Some(color);
        self
    }
    fn set_inactive(&mut self) -> &mut Self {
        self.as_mut().widget.widget_mut().props.insert(Property::Inactive);
        for child in &mut self.as_mut().children {
            child.widget.widget_mut().props.insert(Property::Inactive);
        }
        self
    }
    fn add_child<C: BuildWidget>(&mut self, widget: C) -> &mut Self {
        self.as_mut().children.push(widget.build());
        self
    }
    fn no_container(&mut self) -> &mut Self {
        self.as_mut().widget.widget_mut().container = None;
        self
    }
    fn set_container<C: LayoutContainer + 'static>(&mut self, container: C) -> &mut Self {
        self.as_mut().widget.widget_mut().container = Some(Rc::new(RefCell::new(Box::new(container))));
        self
    }
    fn set_padding(&mut self, padding: f64) -> &mut Self {
        {
            let mut widget = self.as_mut().widget.widget_mut();
            widget.container.as_mut().unwrap().borrow_mut().set_padding(padding);
        }
        self
    }
    fn layout(&mut self) -> LayoutGuard {
        self.as_mut().widget.layout()
    }
    fn id(&mut self) -> WidgetId {
        self.as_mut().widget.widget_mut().id
    }
}

pub struct LayoutGuard<'a> {
    guard: RefMut<'a, Widget>
}
impl<'b> Deref for LayoutGuard<'b> {
    type Target = Layout;
    fn deref(&self) -> &Layout {
        &self.guard.layout
    }
}
impl<'b> DerefMut for LayoutGuard<'b> {
    fn deref_mut(&mut self) -> &mut Layout {
        &mut self.guard.layout
    }
}

pub struct PropsGuard<'a> {
    guard: RefMut<'a, Widget>
}
impl<'b> Deref for PropsGuard<'b> {
    type Target = PropSet;
    fn deref(&self) -> &PropSet {
        &self.guard.props
    }
}
impl<'b> DerefMut for PropsGuard<'b> {
    fn deref_mut(&mut self) -> &mut PropSet {
        &mut self.guard.props
    }
}

pub struct DrawableGuard<'a> {
    guard: RefMut<'a, Widget>
}
impl<'a> DrawableGuard<'a> {
    pub fn downcast_ref<T: Drawable>(&self) -> Option<&T> {
        if let Some(ref drawable) = self.guard.drawable {
            drawable.drawable.as_ref().downcast_ref::<T>()
        } else {
            None
        }
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
        let widget = Widget {
            id: resources().widget_id(),
            drawable: None,
            props: PropSet::new(),
            layout: Layout::new(None),
            has_updated: false,
            bounds: Rect::zero(),
            debug_name: name,
            debug_color: None,
            children: Vec::new(),
            parent: None,
            container: Some(Rc::new(RefCell::new(Box::new(Frame::new())))),
            handlers: HashMap::new(),
        };
        let mut builder = WidgetBuilder {
            children: Vec::new(),
            widget: WidgetRef::new(widget),
        };
        builder.add_handler_fn(property::prop_change_handle);
        builder
    }

    pub fn build(mut self) -> (Vec<WidgetBuilder>, WidgetRef) {
        self.widget.apply_style();
        (self.children, self.widget)
    }
}

#[derive(Clone)]
pub struct WidgetRef(pub Rc<RefCell<Widget>>);
#[derive(Clone)]
pub struct WidgetRefWeak(pub Weak<RefCell<Widget>>);

impl PartialEq for WidgetRef {
    fn eq(&self, other: &WidgetRef) -> bool {
        self.id() == other.id()
    }
}
impl Eq for WidgetRef {}
impl Hash for WidgetRef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}
use std::fmt;
impl fmt::Debug for WidgetRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.widget().debug_name.clone().unwrap_or("None".to_owned()))
    }
}

impl WidgetRef {
    fn new(widget: Widget) -> Self {
        let widget_ref = WidgetRef(Rc::new(RefCell::new(widget)));
        event!(Target::Ui, ::ui::RegisterWidget(widget_ref.clone()));
        widget_ref
    }
    pub fn widget_mut(&self) -> RefMut<Widget> {
        self.0.borrow_mut()
    }
    pub fn widget(&self) -> Ref<Widget> {
        self.0.borrow()
    }
    pub fn layout(&mut self) -> LayoutGuard {
        LayoutGuard { guard: self.0.borrow_mut() }
    }
    pub fn props(&mut self) -> PropsGuard {
        PropsGuard { guard: self.0.borrow_mut() }
    }
    pub fn drawable(&mut self) -> DrawableGuard {
        DrawableGuard { guard: self.0.borrow_mut() }
    }
    pub fn downgrade(&self) -> WidgetRefWeak {
        WidgetRefWeak(Rc::downgrade(&self.0))
    }
    pub fn id(&self) -> WidgetId {
        self.0.borrow().id
    }
    pub fn debug_name(&self) -> Option<String> {
        self.0.borrow().debug_name.clone()
    }
    pub fn debug_color(&self) -> Option<Color> {
        self.0.borrow().debug_color
    }
    pub fn has_updated(&self) -> bool {
        self.0.borrow().has_updated
    }
    pub fn set_updated(&self, has_updated: bool) {
        self.0.borrow_mut().has_updated = has_updated;
    }
    pub fn bounds(&self) -> Rect {
        self.0.borrow().bounds
    }

    pub fn update<F, T: Drawable + 'static>(&mut self, f: F)
        where F: FnOnce(&mut T)
    {
        let mut widget = self.0.borrow_mut();
        widget.update(f);
    }
    pub fn update_layout<F>(&mut self, f: F)
        where F: FnOnce(&mut Layout)
    {
        f(&mut self.layout());
        event!(Target::Ui, UpdateLayout(self.clone()));
    }

    pub fn apply_style(&mut self) {
        self.0.borrow_mut().apply_style();
    }

    pub fn add_child(&mut self, mut child: WidgetRef) {
        event!(Target::Ui, ::layout::UpdateLayout(child.clone()));
        child.widget_mut().parent = Some(self.downgrade());
        self.widget_mut().children.push(child.clone());
        let mut container = self.widget_mut().container.clone();
        if let Some(ref mut container) = container {
            let mut container = container.borrow_mut();
            container.add_child(self.clone(), child.clone());
        }
        self.event(::ui::WidgetAttachedEvent);
        self.event(::ui::ChildAttachedEvent(self.id(), child.layout().vars.clone()));
    }

    pub fn remove_child(&mut self, child_ref: WidgetRef) {
        let child_id = child_ref.id();
        let mut container = self.widget_mut().container.clone();
        if let Some(ref mut container) = container {
            let mut container = container.borrow_mut();
            container.remove_child(self.clone(), child_id);
        }
        let mut widget = self.widget_mut();
        if let Some(index) = widget.children.iter().position(|widget| widget.id() == child_id) {
            widget.children.remove(index);
        }
        child_ref.event(::ui::WidgetDetachedEvent);
        event!(Target::Ui, ::ui::RemoveWidget(child_ref.clone()));
    }

    pub fn remove_widget(&mut self) {
        if let Some(mut parent) = self.parent() {
            parent.remove_child(self.clone());
        }
    }

    pub fn parent(&mut self) -> Option<WidgetRef> {
        let parent = self.widget().parent.clone();
        parent.unwrap().upgrade()
    }

    pub fn event<T: 'static>(&self, data: T) {
        event!(Target::Widget(self.clone()), data);
    }
    pub fn event_subtree<T: 'static>(&self, data: T) {
        event!(Target::SubTree(self.clone()), data);
    }
    pub fn event_bubble_up<T: 'static>(&self, data: T) {
        event!(Target::BubbleUp(self.clone()), data);
    }
    pub fn trigger_event(&self,
                         type_id: TypeId,
                         event: &Box<Any>)
                         -> bool {
        let handlers = {
            let mut widget = self.0.borrow_mut();
            let mut handlers: Vec<Rc<RefCell<WidgetHandlerWrapper>>> = Vec::new();
            if let Some(event_handlers) = widget.handlers.get_mut(&type_id) {
                for handler in event_handlers {
                    handlers.push(handler.clone());
                }
            }
            handlers
        };

        let mut handled = false;
        for event_handler in handlers {
            // will panic in the case of circular handler calls
            let mut handler = event_handler.borrow_mut();
            let event_args = WidgetEventArgs {
                widget: self.clone(),
                handled: &mut handled,
            };
            handler.handle(event, event_args);
        }
        handled
    }
}
impl WidgetRefWeak {
    pub fn upgrade(&self) -> Option<WidgetRef> {
        if let Some(widget_ref) = self.0.upgrade() {
            Some(WidgetRef(widget_ref))
        } else {
            None
        }
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
    pub children: Vec<WidgetRef>,
    pub parent: Option<WidgetRefWeak>,
    pub container: Option<Rc<RefCell<Box<LayoutContainer>>>>,
    pub handlers: HashMap<TypeId, Vec<Rc<RefCell<WidgetHandlerWrapper>>>>,
}

impl Widget {
    pub fn new(id: WidgetId,
               drawable: Option<DrawableWrapper>,
               props: PropSet,
               layout: Layout,
               debug_name: Option<String>,
               debug_color: Option<Color>,
               container: Option<Rc<RefCell<Box<LayoutContainer>>>>,
               handlers: HashMap<TypeId, Vec<Rc<RefCell<WidgetHandlerWrapper>>>>)
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
            children: Vec::new(),
            parent: None,
            container: container,
            handlers: handlers,
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
        if let Some(crop_to) = crop_to.intersection(&self.bounds) {
            for child in &self.children {
                let mut child = child.widget_mut();
                child.draw(crop_to, glyph_cache, context, graphics);
            }
        }
    }
    pub fn draw_debug(&mut self, context: Context, graphics: &mut G2d) {
        let color = self.debug_color.unwrap_or(::color::GREEN);
        util::draw_rect_outline(self.bounds, color, context, graphics);
        for child in &self.children {
            let mut child = child.widget_mut();
            child.draw_debug(context, graphics);
        }
    }

    pub fn is_mouse_over(&self, mouse: Point) -> bool {
        self.bounds.contains(&mouse)
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
    pub fn drawable<T: Drawable>(&self) -> Option<&T> {
        if let Some(ref drawable) = self.drawable {
            drawable.drawable.as_ref().downcast_ref::<T>()
        } else {
            None
        }
    }
    pub fn debug_constraints(&self) {
        self.layout.debug_constraints();
        for child in &self.children {
            child.widget().debug_constraints();
        }
    }
}
