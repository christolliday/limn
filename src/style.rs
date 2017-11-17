use std::any::{Any, TypeId};
use std::collections::HashMap;

use widget::WidgetBuilder;
use resources::WidgetId;
use widget::draw::{DrawWrapper, Draw};
use widget::property::PropSet;

pub struct Theme {
    type_styles: HashMap<TypeId, Box<MergeStyle + Send>>,
    style_classes: HashMap<(TypeId, String), Box<MergeStyle + Send>>,
    widget_styles: HashMap<(TypeId, WidgetId), Box<MergeStyle + Send>>,
    class_prop_styles: HashMap<(TypeId, String, PropSet), Box<MergeStyle + Send>>,
    widget_prop_styles: HashMap<(TypeId, WidgetId, PropSet), Box<MergeStyle + Send>>,
}

impl Theme {
    pub fn new() -> Self {
        Theme {
            type_styles: HashMap::new(),
            style_classes: HashMap::new(),
            widget_styles: HashMap::new(),
            class_prop_styles: HashMap::new(),
            widget_prop_styles: HashMap::new(),
        }
    }

    pub fn register_type_style<S: Draw + Component + 'static, T: ComponentStyle<Component = S> + Send + 'static>(&mut self, style: T) {
        self.type_styles.insert(TypeId::of::<T>(), Box::new(style));
    }
    pub fn register_style_class<S: Draw + Component + 'static, T: ComponentStyle<Component = S> + Send + 'static>(&mut self, class: &str, style: T) {
        self.style_classes.insert((TypeId::of::<T>(), class.to_owned()), Box::new(style));
    }
    pub fn register_widget_style<S: Draw + Component + 'static, T: ComponentStyle<Component = S> + Send + 'static>(&mut self, widget_id: WidgetId, style: T) {
        self.widget_styles.insert((TypeId::of::<T>(), widget_id), Box::new(style));
    }
    pub fn register_style_class_prop<S: Draw + Component + 'static, T: ComponentStyle<Component = S> + Send + 'static>(&mut self, class: &str, props: PropSet, style: T) {
        self.class_prop_styles.insert((TypeId::of::<T>(), class.to_owned(), props), Box::new(style));
    }
    pub fn register_style_widget_prop<S: Draw + Component + 'static, T: ComponentStyle<Component = S> + Send + 'static>(&mut self, widget_id: WidgetId, props: PropSet, style: T) {
        self.widget_prop_styles.insert((TypeId::of::<T>(), widget_id, props), Box::new(style));
    }

    pub fn get_style(&self, type_id: TypeId, class: Option<String>, widget_id: WidgetId) -> DrawWrapper {
        let mut style = self.type_styles.get(&type_id).unwrap().clone();
        if let Some(class) = class {
            if let Some(class_style) = self.style_classes.get(&(type_id, class)) {
                style = class_style.merge(&style);
            }
        }
        if let Some(widget_style) = self.widget_styles.get(&(type_id, widget_id)) {
            style = widget_style.merge(&style);
        }
        style.wrapper()
    }
}

pub trait Component: Clone {
    fn name() -> String;
}

pub trait ComponentStyle: Clone + 'static {
    type Component: Sized;
    fn merge(&self, other: &Self) -> Self;
    fn component(self) -> Self::Component;
}

pub trait WidgetModifier {
    fn apply(&self, widget: &mut WidgetBuilder);
}

impl <T: Component + 'static> ComponentStyle for T {
    type Component = T;
    fn merge(&self, _: &Self) -> Self {
        self.clone()
    }
    fn component(self) -> Self::Component {
        self
    }
}

pub trait MergeStyle {
    fn merge(&self, lower: &Any) -> Box<MergeStyle + Send>;
    fn wrapper(self: Box<Self>) -> DrawWrapper;
    fn box_clone(&self) -> Box<MergeStyle + Send>;
    fn as_any(&self) -> &Any;
}

impl Clone for Box<MergeStyle + Send> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}

impl <T: Draw + Component + 'static, C: ComponentStyle<Component = T> + Send> MergeStyle for C {
    fn merge(&self, lower: &Any) -> Box<MergeStyle + Send> {
        if let Some(lower) = lower.downcast_ref::<C>() {
            Box::new(self.merge(lower))
        } else {
            Box::new(self.merge(self))
        }
    }
    fn wrapper(self: Box<Self>) -> DrawWrapper {
        DrawWrapper::new(self.component())
    }
    fn box_clone(&self) -> Box<MergeStyle + Send> {
        Box::new((*self).clone())
    }
    fn as_any(&self) -> &Any {
        self
    }
}
