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

use render::RenderBuilder;
use event::{WidgetEventHandler, WidgetEventArgs, WidgetHandlerWrapper};
use layout::{Layout, LayoutVars, LayoutRef};
use layout::container::{LayoutContainer, Frame};
use resources::{resources, WidgetId};
use util::{self, Point, Rect, RectExt};
use color::Color;
use event::Target;
use layout::UpdateLayout;

use self::property::{PropSet, Property};
use self::drawable::{Drawable, DrawableWrapper};
use self::style::Style;

impl AsMut<WidgetRef> for WidgetRef {
    fn as_mut(&mut self) -> &mut WidgetRef {
        self
    }
}
impl AsRef<WidgetRef> for WidgetRef {
    fn as_ref(&self) -> &WidgetRef {
        self
    }
}
pub trait BuildWidget {
    fn build(self) -> WidgetBuilder;
}
impl BuildWidget for WidgetBuilder {
    fn build(self) -> WidgetBuilder {
        self
    }
}
impl LayoutRef for WidgetBuilder {
    fn layout_ref(&self) -> LayoutVars {
        self.widget.widget_mut().layout.vars.clone()
    }
}
impl LayoutRef for WidgetRef {
    fn layout_ref(&self) -> LayoutVars {
        self.widget_mut().layout.vars.clone()
    }
}

#[macro_export]
macro_rules! widget_wrapper {
    ($builder_type:ty) => {
        widget_builder!($builder_type);
        impl $crate::widget::BuildWidget for $builder_type {
            fn build(self) -> WidgetBuilder {
                self.widget
            }
        }
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
        impl ::std::ops::Deref for $builder_type {
            type Target = WidgetBuilder;
            fn deref(&self) -> &WidgetBuilder {
                &self.widget
            }
        }
        impl ::std::ops::DerefMut for $builder_type {
            fn deref_mut(&mut self) -> &mut WidgetBuilder {
                &mut self.widget
            }
        }
        impl $crate::layout::LayoutRef for $builder_type {
            fn layout_ref(&self) -> $crate::layout::LayoutVars {
                self.as_ref().widget.layout_vars()
            }
        }
    };
}

pub struct LayoutGuard<'a> {
    guard: Ref<'a, Widget>
}
impl<'b> Deref for LayoutGuard<'b> {
    type Target = Layout;
    fn deref(&self) -> &Layout {
        &self.guard.layout
    }
}

pub struct LayoutGuardMut<'a> {
    guard: RefMut<'a, Widget>
}
impl<'b> Deref for LayoutGuardMut<'b> {
    type Target = Layout;
    fn deref(&self) -> &Layout {
        &self.guard.layout
    }
}
impl<'b> DerefMut for LayoutGuardMut<'b> {
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

pub struct WidgetBuilder {
    pub widget: WidgetRef,
}

impl WidgetBuilder {
    pub fn new(name: &str) -> Self {
        let mut widget = WidgetBuilder {
            widget: WidgetRef::new(Widget::new(name.to_owned())),
        };
        widget.add_handler_fn(property::prop_change_handle);
        widget
    }
    pub fn widget_ref(&self) -> WidgetRef {
        self.widget.clone()
    }
    pub fn id(&self) -> WidgetId {
        self.widget.id()
    }
    pub fn set_drawable<T: Drawable + 'static>(&mut self, drawable: T) -> &mut Self {
        self.widget.widget_mut().drawable = Some(DrawableWrapper::new(drawable));
        self
    }
    pub fn set_drawable_with_style<T: Drawable + 'static, S: Style<T> + 'static>(&mut self, drawable: T, style: S) -> &mut Self {
        self.widget.widget_mut().drawable = Some(DrawableWrapper::new_with_style(drawable, style));
        self.widget.apply_style();
        self
    }
    pub fn add_handler<E: 'static, T: WidgetEventHandler<E> + 'static>(&mut self, handler: T) -> &mut Self {
        self.add_handler_wrapper(TypeId::of::<E>(), WidgetHandlerWrapper::new(handler))
    }
    pub fn add_handler_fn<E: 'static, T: Fn(&E, WidgetEventArgs) + 'static>(&mut self, handler: T) -> &mut Self {
        self.add_handler_wrapper(TypeId::of::<E>(), WidgetHandlerWrapper::new_from_fn(handler))
    }
    fn add_handler_wrapper(&mut self, type_id: TypeId, handler: WidgetHandlerWrapper) -> &mut Self {
        self.widget.widget_mut().handlers.entry(type_id).or_insert_with(Vec::new)
            .push(Rc::new(RefCell::new(handler)));
        self
    }
    pub fn set_inactive(&mut self) -> &mut Self {
        self.widget.widget_mut().props.insert(Property::Inactive);
        for child in &mut self.widget.widget_mut().children {
            child.widget_mut().props.insert(Property::Inactive);
        }
        self
    }
    pub fn no_container(&mut self) -> &mut Self {
        self.widget.widget_mut().container = None;
        self
    }
    pub fn set_container<C: LayoutContainer + 'static>(&mut self, container: C) -> &mut Self {
        self.widget.widget_mut().container = Some(Rc::new(RefCell::new(Box::new(container))));
        self
    }
    pub fn set_padding(&mut self, padding: f32) -> &mut Self {
        {
            let mut widget = self.widget.widget_mut();
            widget.container.as_mut().unwrap().borrow_mut().set_padding(padding);
        }
        self
    }
    pub fn layout(&mut self) -> LayoutGuardMut {
        LayoutGuardMut { guard: self.widget.0.borrow_mut() }
    }
    pub fn add_child<U: BuildWidget>(&mut self, child: U) -> &mut Self {
        self.widget.add_child(child);
        self
    }
    pub fn set_name(&mut self, name: &str) -> &mut Self {
        self.widget.widget_mut().name = name.to_owned();
        self.widget.widget_mut().layout.name = Some(name.to_owned());
        self
    }
}

#[derive(Clone)]
pub struct WidgetRef(pub Rc<RefCell<Widget>>);
#[derive(Clone)]
pub struct WidgetWeak(pub Weak<RefCell<Widget>>);

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
        write!(f, "{}", self.widget().name)
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
        LayoutGuard { guard: self.0.borrow() }
    }
    pub fn layout_vars(&self) -> LayoutVars {
        self.0.borrow().layout.vars.clone()
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
    pub fn set_name(&mut self, name: &str) -> &mut Self {
        self.widget_mut().name = name.to_owned();
        self.widget_mut().layout.name = Some(name.to_owned());
        event!(Target::Ui, UpdateLayout(self.clone()));
        self
    }
    pub fn set_debug_color(&mut self, color: Color) -> &mut Self {
        self.widget_mut().debug_color = Some(color);
        self
    }
    pub fn name(&self) -> String {
        self.0.borrow().name.clone()
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
        self.0.borrow_mut().update(f);
        self.event(self::style::StyleUpdated);
    }
    pub fn update_layout<F>(&mut self, f: F)
        where F: FnOnce(&mut Layout)
    {
        let layout = &mut self.0.borrow_mut().layout;
        f(layout);
        event!(Target::Ui, UpdateLayout(self.clone()));
    }

    pub fn apply_style(&mut self) {
        self.0.borrow_mut().apply_style();
        self.event(self::style::StyleUpdated);
    }

    pub fn add_child<U: BuildWidget>(&mut self, child: U) -> &mut Self {
        let mut child = child.build();
        event!(Target::Ui, ::layout::UpdateLayout(child.widget_ref()));
        child.widget.widget_mut().parent = Some(self.downgrade());
        child.widget.apply_style();
        self.widget_mut().children.push(child.widget_ref());
        let mut container = self.widget_mut().container.clone();
        if let Some(ref mut container) = container {
            let mut container = container.borrow_mut();
            container.add_child(self.clone(), child.widget_ref());
        }
        self.update_layout(|layout| layout.add_child(child.id().0));
        self.event(::ui::WidgetAttachedEvent);
        self.event(::ui::ChildAttachedEvent(self.id(), child.layout().vars.clone()));
        self
    }

    pub fn remove_child(&mut self, child_ref: WidgetRef) {
        let child_id = child_ref.id();
        let mut container = self.widget_mut().container.clone();
        if let Some(ref mut container) = container {
            let mut container = container.borrow_mut();
            container.remove_child(self.clone(), child_id);
        }
        self.update_layout(|layout| layout.remove_child(child_id.0));
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
    pub name: String,
    pub debug_color: Option<Color>,
    pub children: Vec<WidgetRef>,
    pub parent: Option<WidgetWeak>,
    pub container: Option<Rc<RefCell<Box<LayoutContainer>>>>,
    pub handlers: HashMap<TypeId, Vec<Rc<RefCell<WidgetHandlerWrapper>>>>,
}

impl Widget {
    fn new(name: String) -> Self {
        let id = resources().widget_id();
        Widget {
            id: id,
            drawable: None,
            props: PropSet::new(),
            layout: Layout::new(id.0, Some(name.clone())),
            has_updated: false,
            bounds: Rect::zero(),
            name: name,
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
    pub fn draw(&mut self, crop_to: Rect, renderer: &mut RenderBuilder) {
        let bounds = self.bounds;
        let clip_id = renderer.builder.define_clip(None, bounds.typed(), vec![], None);
        renderer.builder.push_clip_id(clip_id);
        if let Some(drawable) = self.drawable.as_mut() {
            drawable.drawable.draw(bounds, crop_to, renderer);
        }
        if let Some(crop_to) = crop_to.intersection(&bounds) {
            for child in &self.children {
                let mut child = child.widget_mut();
                child.draw(crop_to, renderer);
            }
        }
        renderer.builder.pop_clip_id();
    }
    pub fn draw_debug(&mut self, renderer: &mut RenderBuilder) {
        let color = self.debug_color.unwrap_or(::color::GREEN);
        util::draw_rect_outline(self.bounds, color, renderer);
        for child in &self.children {
            child.widget_mut().draw_debug(renderer);
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
}
