use std::fmt::Debug;
use std::any::{Any, TypeId};
use std::collections::HashMap;

use widget::WidgetBuilder;
use resources::WidgetId;
use widget::draw::{DrawWrapper, Draw};
use widget::property::PropSet;

use linked_hash_map::LinkedHashMap;

pub struct Theme {
    type_styles: HashMap<TypeId, Box<MergeStyle + Send>>,
    style_classes: HashMap<(TypeId, String), Box<MergeStyle + Send>>,
    widget_styles: HashMap<(TypeId, WidgetId), Box<MergeStyle + Send>>,
    class_style_selector: HashMap<(TypeId, String), LinkedHashMap<PropSet, Box<MergeStyle + Send>>>,
    widget_style_selector: HashMap<(TypeId, WidgetId), LinkedHashMap<PropSet, Box<MergeStyle + Send>>>,
}

impl Theme {
    pub fn new() -> Self {
        Theme {
            type_styles: HashMap::new(),
            style_classes: HashMap::new(),
            widget_styles: HashMap::new(),
            class_style_selector: HashMap::new(),
            widget_style_selector: HashMap::new(),
        }
    }

    pub fn register_type_style<S: Draw + Component + 'static, T: ComponentStyle<Component = S> + Debug + Send + 'static>(&mut self, style: T) {
        self.type_styles.insert(TypeId::of::<T>(), Box::new(style));
    }
    pub fn register_style_class<S: Draw + Component + 'static, T: ComponentStyle<Component = S> + Debug + Send + 'static>(&mut self, class: &str, style: T) {
        self.style_classes.insert((TypeId::of::<T>(), class.to_owned()), Box::new(style));
    }
    pub fn register_widget_style<S: Draw + Component + 'static, T: ComponentStyle<Component = S> + Debug + Send + 'static>(&mut self, widget_id: WidgetId, style: T) {
        self.widget_styles.insert((TypeId::of::<T>(), widget_id), Box::new(style));
    }
    pub fn register_style_class_prop<S: Draw + Component + 'static, T: ComponentStyle<Component = S> + Debug + Send + 'static>(&mut self, class: &str, props: PropSet, style: T) {
        self.class_style_selector.entry((TypeId::of::<T>(), class.to_owned())).or_insert_with(LinkedHashMap::new).insert(props, Box::new(style));
    }
    pub fn register_style_widget_prop<S: Draw + Component + 'static, T: ComponentStyle<Component = S> + Debug + Send + 'static>(&mut self, widget_id: WidgetId, props: PropSet, style: T) {
        self.widget_style_selector.entry((TypeId::of::<T>(), widget_id)).or_insert_with(LinkedHashMap::new).insert(props, Box::new(style));
    }

    pub fn get_style(&self, type_id: TypeId, class: Option<String>, widget_id: WidgetId, props: PropSet) -> DrawWrapper {
        let mut style = self.type_styles.get(&type_id).unwrap().clone();
        if let Some(class) = class {
            if let Some(class_style) = self.style_classes.get(&(type_id, class.clone())) {
                style = class_style.clone().merge(style.clone());
            }
            if let Some(selector) = self.class_style_selector.get(&(type_id, class)) {
                style = selector.select(style, &props);
            }
        }
        if let Some(widget_style) = self.widget_styles.get(&(type_id, widget_id)) {
            style = widget_style.clone().merge(style.clone());
        }
        if let Some(selector) = self.widget_style_selector.get(&(type_id, widget_id)) {
            style = selector.select(style, &props);
        }
        style.wrapper()
    }
}

trait Selector {
    fn select(&self, style: Box<MergeStyle + Send>, props: &PropSet) -> Box<MergeStyle + Send>;
}
impl Selector for LinkedHashMap<PropSet, Box<MergeStyle + Send>> {
    fn select(&self, mut style: Box<MergeStyle + Send>, props: &PropSet) -> Box<MergeStyle + Send> {
        if self.contains_key(&props) {
            style = self.get(&props).unwrap().clone().merge(style.clone());
        } else {
            if let Some(new_style) = self.iter().find(|&(matcher_props, _)| {
                matcher_props.is_subset(&props)
            }).map(|(_, val)| val) {
                style = new_style.clone().merge(style.clone());
            }
        }
        style
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

pub trait MergeStyle: Debug {
    fn merge(self: Box<Self>, lower: Box<MergeStyle + Send>) -> Box<MergeStyle + Send>;
    fn wrapper(self: Box<Self>) -> DrawWrapper;
    fn box_clone(&self) -> Box<MergeStyle + Send>;
    fn as_any(&self) -> &Any;
}

impl Clone for Box<MergeStyle + Send> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}

impl <T: Draw + Component + 'static, C: ComponentStyle<Component = T> + Debug + Send> MergeStyle for C {
    fn merge(self: Box<Self>, lower: Box<MergeStyle + Send>) -> Box<MergeStyle + Send> {
        let upper = self.as_any().downcast_ref::<C>().unwrap();
        let lower = lower.as_any().downcast_ref::<C>().unwrap();
        Box::new(upper.merge(lower))
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
