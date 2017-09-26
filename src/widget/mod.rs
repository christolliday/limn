#[macro_use]
pub mod style;
pub mod property;
pub mod draw;

use std::any::{TypeId, Any};
use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::cell::{RefCell, Ref, RefMut};
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use std::fmt;

use render::RenderBuilder;
use event::{self, EventHandler, EventArgs, EventHandlerWrapper};
use layout::{Layout, LayoutVars, LayoutRef};
use ui::Ui;
use resources::{resources, WidgetId};
use util::{Point, Rect, RectExt};
use render;
use color::Color;
use event::Target;
use layout::UpdateLayout;

use self::property::{PropSet, Property};
use self::draw::{Draw, DrawWrapper};
use self::style::Style;

#[derive(Clone)]
pub struct WidgetRef(pub Rc<RefCell<Widget>>);

impl WidgetRef {
    fn new(widget: Widget) -> Self {
        let widget_ref = WidgetRef(Rc::new(RefCell::new(widget)));
        event::event(Target::Root, ::ui::RegisterWidget(widget_ref.clone()));
        widget_ref
    }
    pub fn widget_mut(&self) -> RefMut<Widget> {
        self.0.borrow_mut()
    }
    pub fn widget(&self) -> Ref<Widget> {
        self.0.borrow()
    }
    pub fn add_handler<E: 'static, T: EventHandler<E> + 'static>(&mut self, handler: T) -> &mut Self {
        self.add_handler_wrapper(TypeId::of::<E>(), EventHandlerWrapper::new(handler))
    }
    pub fn add_handler_fn<E: 'static, T: Fn(&E, EventArgs) + 'static>(&mut self, handler: T) -> &mut Self {
        self.add_handler_wrapper(TypeId::of::<E>(), EventHandlerWrapper::new_from_fn(handler))
    }
    fn add_handler_wrapper(&mut self, type_id: TypeId, handler: EventHandlerWrapper) -> &mut Self {
        self.widget_mut().handlers.entry(type_id).or_insert_with(Vec::new)
            .push(Rc::new(RefCell::new(handler)));
        self
    }
    pub fn layout(&mut self) -> LayoutGuard {
        LayoutGuard { guard: self.0.borrow() }
    }
    pub fn layout_vars(&self) -> LayoutVars {
        self.0.borrow().layout.vars.clone()
    }
    pub fn props(&self) -> PropsGuard {
        PropsGuard { guard: self.0.borrow() }
    }
    pub fn add_prop(&mut self, property: Property) {
        self.0.borrow_mut().props.insert(property);
        for mut child in self.children() {
            child.add_prop(property);
        }
        self.apply_style();
    }
    pub fn remove_prop(&mut self, property: Property) {
        self.0.borrow_mut().props.remove(&property);
        for mut child in self.children() {
            child.remove_prop(property);
        }
        self.apply_style();
    }
    pub fn draw_state(&mut self) -> DrawStateGuard {
        DrawStateGuard { guard: self.0.borrow_mut() }
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
        event::event(Target::Root, UpdateLayout(self.clone()));
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

    pub fn update<F, T: Draw + 'static>(&mut self, f: F)
        where F: FnOnce(&mut T)
    {
        self.0.borrow_mut().update(f);
        self.event(self::style::StyleUpdated);
    }
    pub fn update_layout<F>(&self, f: F)
        where F: FnOnce(&mut Layout)
    {
        let layout = &mut self.0.borrow_mut().layout;
        f(layout);
        event::event(Target::Root, UpdateLayout(self.clone()));
    }

    pub fn apply_style(&mut self) {
        if self.0.borrow_mut().apply_style() {
            self.event(self::style::StyleUpdated);
        }
    }

    pub fn add_child<U: Into<WidgetRef>>(&mut self, child: U) -> &mut Self {
        let mut child = child.into();
        event::event(Target::Root, ::layout::UpdateLayout(child.clone()));
        child.widget_mut().parent = Some(self.downgrade());
        child.widget_mut().props.extend(self.props().iter().cloned());
        child.apply_style();
        self.widget_mut().children.push(child.clone());
        self.update_layout(|layout| {
            child.update_layout(|child_layout| {
                layout.add_child(child_layout);
            });
        });
        self.event(::ui::WidgetAttachedEvent);
        self.event(::ui::ChildAttachedEvent(self.id(), child.layout().vars.clone()));
        self.event(::ui::ChildrenUpdatedEvent::Added(child));
        self
    }

    pub fn remove_child(&mut self, child_ref: WidgetRef) {
        let child_id = child_ref.id();
        self.update_layout(|layout| {
            child_ref.update_layout(|child_layout| {
                layout.remove_child(child_layout);
            });
        });
        let mut widget = self.widget_mut();
        if let Some(index) = widget.children.iter().position(|widget| widget.id() == child_id) {
            widget.children.remove(index);
        }
        self.event(::ui::ChildrenUpdatedEvent::Removed(child_ref.clone()));
        child_ref.event(::ui::WidgetDetachedEvent);
        event::event(Target::Root, ::ui::RemoveWidget(child_ref.clone()));
    }

    pub fn remove_widget(&mut self) {
        if let Some(mut parent) = self.parent() {
            parent.remove_child(self.clone());
        }
    }

    pub fn parent(&self) -> Option<WidgetRef> {
        self.widget().parent.as_ref().and_then(|parent| parent.upgrade())
    }

    pub fn children(&self) -> Vec<WidgetRef> {
        self.widget().children.clone()
    }

    pub fn event<T: 'static>(&self, data: T) {
        event::event(Target::Widget(self.clone()), data);
    }
    pub fn event_subtree<T: 'static>(&self, data: T) {
        event::event(Target::SubTree(self.clone()), data);
    }
    pub fn event_bubble_up<T: 'static>(&self, data: T) {
        event::event(Target::BubbleUp(self.clone()), data);
    }
    pub fn trigger_event(&self, ui: &mut Ui, type_id: TypeId, event: &Any) -> bool {
        let handlers = {
            let mut widget = self.0.borrow_mut();
            let mut handlers: Vec<Rc<RefCell<EventHandlerWrapper>>> = Vec::new();
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
            let event_args = EventArgs {
                widget: self.clone(),
                ui: ui,
                handled: &mut handled,
            };
            handler.handle(event, event_args);
        }
        handled
    }
}

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

impl ::std::fmt::Debug for WidgetRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.widget().name)
    }
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

impl LayoutRef for WidgetRef {
    fn layout_ref(&self) -> LayoutVars {
        self.layout_vars()
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
    guard: Ref<'a, Widget>
}

impl<'b> Deref for PropsGuard<'b> {
    type Target = PropSet;
    fn deref(&self) -> &PropSet {
        &self.guard.props
    }
}

pub struct DrawStateGuard<'a> {
    guard: RefMut<'a, Widget>
}

impl<'a> DrawStateGuard<'a> {
    pub fn downcast_ref<T: Draw>(&self) -> Option<&T> {
        if let Some(ref draw_state) = self.guard.draw_state {
            draw_state.state.as_ref().downcast_ref::<T>()
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct WidgetWeak(pub Weak<RefCell<Widget>>);

impl WidgetWeak {
    pub fn upgrade(&self) -> Option<WidgetRef> {
        if let Some(widget_ref) = self.0.upgrade() {
            Some(WidgetRef(widget_ref))
        } else {
            None
        }
    }
}

/// Internal Widget representation, usually handled through a WidgetRef
pub struct Widget {
    id: WidgetId,
    draw_state: Option<DrawWrapper>,
    props: PropSet,
    has_updated: bool,
    pub(super) layout: Layout,
    pub(super) bounds: Rect,
    name: String,
    debug_color: Option<Color>,
    children: Vec<WidgetRef>,
    parent: Option<WidgetWeak>,
    handlers: HashMap<TypeId, Vec<Rc<RefCell<EventHandlerWrapper>>>>,
}

impl Widget {
    fn new(name: String) -> Self {
        let id = resources().widget_id();
        Widget {
            id: id,
            draw_state: None,
            props: PropSet::new(),
            layout: Layout::new(id.0, Some(name.clone())),
            has_updated: false,
            bounds: Rect::zero(),
            name: name,
            debug_color: None,
            children: Vec::new(),
            parent: None,
            handlers: HashMap::new(),
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn layout(&mut self) -> &mut Layout {
        &mut self.layout
    }
    pub fn draw(&mut self, crop_to: Rect, renderer: &mut RenderBuilder) {
        let bounds = self.bounds;
        let clip_id = renderer.builder.define_clip(None, bounds.typed(), vec![], None);
        renderer.builder.push_clip_id(clip_id);
        if let Some(draw_state) = self.draw_state.as_mut() {
            draw_state.state.draw(bounds, crop_to, renderer);
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
        render::draw_rect_outline(self.bounds, color, renderer);
        for child in &self.children {
            child.widget_mut().draw_debug(renderer);
        }
    }

    pub fn is_under_cursor(&self, cursor: Point) -> bool {
        if let Some(ref draw_state) = self.draw_state {
            draw_state.is_under_cursor(self.bounds, cursor)
        } else {
            false
        }
    }
    pub fn update<F, T: Draw + 'static>(&mut self, f: F)
        where F: FnOnce(&mut T)
    {
        if let Some(ref mut draw_state) = self.draw_state {
            self.has_updated = true;
            let state = draw_state.state.as_mut().downcast_mut::<T>().expect("Called update on widget with wrong draw_state type");
            f(state);
        }
    }
    fn apply_style(&mut self) -> bool {
        if let Some(ref mut draw_state) = self.draw_state {
            if draw_state.apply_style(&self.props) {
                self.has_updated = true;
                return true;
            }
        }
        false
    }
    pub fn draw_state<T: Draw>(&self) -> Option<&T> {
        if let Some(ref draw_state) = self.draw_state {
            draw_state.state.as_ref().downcast_ref::<T>()
        } else {
            None
        }
    }
}

/// Used to initialize and modify a Widget before it's been added to a parent Widget
pub struct WidgetBuilder {
    pub widget: WidgetRef,
}

impl WidgetBuilder {
    pub fn new(name: &str) -> Self {
        WidgetBuilder {
            widget: WidgetRef::new(Widget::new(name.to_owned())),
        }
    }
    pub fn widget_ref(&self) -> WidgetRef {
        self.widget.clone()
    }
    pub fn id(&self) -> WidgetId {
        self.widget.id()
    }
    pub fn set_draw_state<T: Draw + 'static>(&mut self, draw_state: T) -> &mut Self {
        self.widget.widget_mut().draw_state = Some(DrawWrapper::new(draw_state));
        self.widget.widget_mut().apply_style();
        self.widget.event(self::style::StyleUpdated);
        self
    }
    pub fn set_draw_state_with_style<T: Draw + 'static, S: Style<T> + 'static>(&mut self, draw_state: T, style: S) -> &mut Self {
        self.widget.widget_mut().draw_state = Some(DrawWrapper::new_with_style(draw_state, style));
        self.widget.widget_mut().apply_style();
        self.widget.event(self::style::StyleUpdated);
        self
    }
    pub fn add_handler<E: 'static, T: EventHandler<E> + 'static>(&mut self, handler: T) -> &mut Self {
        self.widget.add_handler(handler);
        self
    }
    pub fn add_handler_fn<E: 'static, T: Fn(&E, EventArgs) + 'static>(&mut self, handler: T) -> &mut Self {
        self.widget.add_handler_fn(handler);
        self
    }
    pub fn add_prop(&mut self, property: Property) -> &mut Self {
        self.widget.widget_mut().props.insert(property);
        for child in &mut self.widget.widget_mut().children {
            child.widget_mut().props.insert(property);
            child.apply_style();
        }
        self
    }
    pub fn layout(&mut self) -> LayoutGuardMut {
        LayoutGuardMut { guard: self.widget.0.borrow_mut() }
    }
    pub fn add_child<U: Into<WidgetRef>>(&mut self, child: U) -> &mut Self {
        self.widget.add_child(child);
        self
    }
    pub fn set_name(&mut self, name: &str) -> &mut Self {
        self.widget.widget_mut().name = name.to_owned();
        self.widget.widget_mut().layout.name = Some(name.to_owned());
        self
    }
}

impl Into<WidgetRef> for WidgetBuilder {
    fn into(mut self) -> WidgetRef {
        self.widget.apply_style();
        self.widget
    }
}

pub trait AsWidgetRef {
    fn widget_ref(&self) -> WidgetRef;
}

impl AsWidgetRef for WidgetBuilder {
    fn widget_ref(&self) -> WidgetRef {
        self.widget.clone()
    }
}

impl LayoutRef for WidgetBuilder {
    fn layout_ref(&self) -> LayoutVars {
        self.widget_ref().layout_vars()
    }
}

#[macro_export]
macro_rules! widget_wrapper {
    ($builder_type:ty) => {
        widget_builder!($builder_type);
        impl Into<$crate::widget::WidgetBuilder> for $builder_type {
            fn into(self) -> WidgetBuilder {
                self.widget
            }
        }
    }
}

#[macro_export]
macro_rules! widget_builder {
    ($builder_type:ty) => {
        impl $crate::widget::AsWidgetRef for $builder_type {
            fn widget_ref(&self) -> $crate::widget::WidgetRef {
                self.widget.widget_ref()
            }
        }
        impl Into<$crate::widget::WidgetRef> for $builder_type {
            fn into(self) -> $crate::widget::WidgetRef {
                let builder: WidgetBuilder = self.into();
                builder.into()
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
                self.widget_ref().layout_vars()
            }
        }
    };
}
