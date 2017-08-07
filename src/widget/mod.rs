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

impl AsMut<Widget> for Widget {
    fn as_mut(&mut self) -> &mut Widget {
        self
    }
}
impl AsRef<Widget> for Widget {
    fn as_ref(&self) -> &Widget {
        self
    }
}
pub trait BuildWidget {
    fn build(self) -> Widget;
}
impl BuildWidget for Widget {
    fn build(self) -> Widget {
        self
    }
}
impl <'a> BuildWidget for &'a mut Widget {
    fn build(self) -> Widget {
        self.clone()
    }
}
impl LayoutRef for Widget {
    fn layout_ref(&self) -> LayoutVars {
        self.widget_mut().layout.vars.clone()
    }
}

#[macro_export]
macro_rules! widget_wrapper {
    ($builder_type:ty) => {
        widget_builder!($builder_type);
        impl $crate::widget::BuildWidget for $builder_type {
            fn build(self) -> Widget {
                self.widget
            }
        }
    }
}

#[macro_export]
macro_rules! widget_builder {
    ($builder_type:ty) => {
        impl AsMut<Widget> for $builder_type {
            fn as_mut(&mut self) -> &mut Widget {
                &mut self.widget
            }
        }
        impl AsRef<Widget> for $builder_type {
            fn as_ref(&self) -> &Widget {
                &self.widget
            }
        }
        impl ::std::ops::Deref for $builder_type {
            type Target = Widget;
            fn deref(&self) -> &Widget {
                &self.widget
            }
        }
        impl ::std::ops::DerefMut for $builder_type {
            fn deref_mut(&mut self) -> &mut Widget {
                &mut self.widget
            }
        }
        impl $crate::layout::LayoutRef for $builder_type {
            fn layout_ref(&self) -> $crate::layout::LayoutVars {
                self.as_ref().widget_mut().layout.vars.clone()
            }
        }
    };
}

pub struct LayoutGuard<'a> {
    guard: RefMut<'a, WidgetInner>
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
    guard: RefMut<'a, WidgetInner>
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
    guard: RefMut<'a, WidgetInner>
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

#[derive(Clone)]
pub struct Widget(pub Rc<RefCell<WidgetInner>>);
#[derive(Clone)]
pub struct WidgetWeak(pub Weak<RefCell<WidgetInner>>);

impl PartialEq for Widget {
    fn eq(&self, other: &Widget) -> bool {
        self.id() == other.id()
    }
}
impl Eq for Widget {}
impl Hash for Widget {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}
use std::fmt;
impl fmt::Debug for Widget {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.widget().debug_name.clone().unwrap_or_else(|| "None".to_owned()))
    }
}

impl Widget {
    pub fn new() -> Self {
        Widget::new_widget(WidgetInner::new(None))
    }
    pub fn new_named(name: &str) -> Self {
        Widget::new_widget(WidgetInner::new(Some(name.to_owned())))
    }
    fn new_widget(widget: WidgetInner) -> Self {
        let mut widget_ref = Widget(Rc::new(RefCell::new(widget)));
        event!(Target::Ui, ::ui::RegisterWidget(widget_ref.clone()));
        widget_ref.add_handler_fn(property::prop_change_handle);
        widget_ref
    }
    pub fn widget_mut(&self) -> RefMut<WidgetInner> {
        self.0.borrow_mut()
    }
    pub fn widget(&self) -> Ref<WidgetInner> {
        self.0.borrow()
    }
    pub fn set_drawable<T: Drawable + 'static>(&mut self, drawable: T) -> &mut Self {
        self.widget_mut().drawable = Some(DrawableWrapper::new(drawable));
        self
    }
    pub fn set_drawable_with_style<T: Drawable + 'static, S: Style<T> + 'static>(&mut self, drawable: T, style: S) -> &mut Self {
        self.widget_mut().drawable = Some(DrawableWrapper::new_with_style(drawable, style));
        self
    }
    pub fn add_handler<E: 'static, T: WidgetEventHandler<E> + 'static>(&mut self, handler: T) -> &mut Self {
        self.add_handler_wrapper(TypeId::of::<E>(), WidgetHandlerWrapper::new(handler))
    }
    pub fn add_handler_fn<E: 'static, T: Fn(&E, WidgetEventArgs) + 'static>(&mut self, handler: T) -> &mut Self {
        self.add_handler_wrapper(TypeId::of::<E>(), WidgetHandlerWrapper::new_from_fn(handler))
    }
    pub fn add_handler_wrapper(&mut self, type_id: TypeId, handler: WidgetHandlerWrapper) -> &mut Self {
        self.widget_mut().handlers.entry(type_id).or_insert_with(Vec::new)
            .push(Rc::new(RefCell::new(handler)));
        self
    }
    pub fn set_inactive(&mut self) -> &mut Self {
        self.widget_mut().props.insert(Property::Inactive);
        for child in &mut self.widget_mut().children {
            child.widget_mut().props.insert(Property::Inactive);
        }
        self
    }
    pub fn no_container(&mut self) -> &mut Self {
        self.widget_mut().container = None;
        self
    }
    pub fn set_container<C: LayoutContainer + 'static>(&mut self, container: C) -> &mut Self {
        self.widget_mut().container = Some(Rc::new(RefCell::new(Box::new(container))));
        self
    }
    pub fn set_padding(&mut self, padding: f64) -> &mut Self {
        {
            let mut widget = self.widget_mut();
            widget.container.as_mut().unwrap().borrow_mut().set_padding(padding);
        }
        self
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
    pub fn downgrade(&self) -> WidgetWeak {
        WidgetWeak(Rc::downgrade(&self.0))
    }
    pub fn id(&self) -> WidgetId {
        self.0.borrow().id
    }
    pub fn set_debug_name(&mut self, name: &str) -> &mut Self {
        self.widget_mut().debug_name = Some(name.to_owned());
        self.widget_mut().layout.name = Some(name.to_owned());
        self
    }
    pub fn set_debug_color(&mut self, color: Color) -> &mut Self {
        self.widget_mut().debug_color = Some(color);
        self
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

    pub fn add_child<U: BuildWidget>(&mut self, child: U) -> &mut Self{
        let mut child = child.build();
        event!(Target::Ui, ::layout::UpdateLayout(child.clone()));
        child.widget_mut().parent = Some(self.downgrade());
        child.apply_style();
        self.widget_mut().children.push(child.clone());
        let mut container = self.widget_mut().container.clone();
        if let Some(ref mut container) = container {
            let mut container = container.borrow_mut();
            container.add_child(self.clone(), child.clone());
        }
        self.event(::ui::WidgetAttachedEvent);
        self.event(::ui::ChildAttachedEvent(self.id(), child.layout().vars.clone()));
        self
    }

    pub fn remove_child(&mut self, child_ref: Widget) {
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

    pub fn parent(&mut self) -> Option<Widget> {
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
    pub fn trigger_event(&self, type_id: TypeId, event: &Any) -> bool {
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
impl WidgetWeak {
    pub fn upgrade(&self) -> Option<Widget> {
        if let Some(widget_ref) = self.0.upgrade() {
            Some(Widget(widget_ref))
        } else {
            None
        }
    }
}

pub struct WidgetInner {
    pub id: WidgetId,
    pub drawable: Option<DrawableWrapper>,
    pub props: PropSet,
    pub has_updated: bool,
    pub layout: Layout,
    pub bounds: Rect,
    pub debug_name: Option<String>,
    pub debug_color: Option<Color>,
    pub children: Vec<Widget>,
    pub parent: Option<WidgetWeak>,
    pub container: Option<Rc<RefCell<Box<LayoutContainer>>>>,
    pub handlers: HashMap<TypeId, Vec<Rc<RefCell<WidgetHandlerWrapper>>>>,
}

impl WidgetInner {
    fn new(name: Option<String>) -> Self {
        WidgetInner {
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
        }
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
