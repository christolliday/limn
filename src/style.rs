use std::fmt::Debug;
use std::any::{Any, TypeId};
use std::collections::HashMap;

use widget::WidgetBuilder;
use resources::WidgetId;
use widget::draw::{DrawWrapper, Draw};
use widget::property::PropSet;

use linked_hash_map::LinkedHashMap;

pub struct Theme {
    type_styles: HashMap<TypeId, Box<DrawMergeStyle>>,
    class_styles: HashMap<(TypeId, String), Box<DrawMergeStyle>>,
    widget_styles: HashMap<(TypeId, WidgetId), Box<DrawMergeStyle>>,
    class_style_selectors: HashMap<(TypeId, String), LinkedHashMap<PropSet, Box<DrawMergeStyle>>>,
    widget_style_selectors: HashMap<(TypeId, WidgetId), LinkedHashMap<PropSet, Box<DrawMergeStyle>>>,
    modifier_type_styles: HashMap<TypeId, Box<ModifierMergeStyle>>,
}

impl Theme {
    pub fn new() -> Self {
        Theme {
            type_styles: HashMap::new(),
            class_styles: HashMap::new(),
            widget_styles: HashMap::new(),
            class_style_selectors: HashMap::new(),
            widget_style_selectors: HashMap::new(),
            modifier_type_styles: HashMap::new(),
        }
    }

    pub fn register_type_style<S: Draw + Component + 'static, T: ComponentStyle<Component = S> + Debug + Send + 'static>(&mut self, style: T) {
        self.type_styles.insert(TypeId::of::<T>(), Box::new(style));
    }
    pub fn register_class_style<S: Draw + Component + 'static, T: ComponentStyle<Component = S> + Debug + Send + 'static>(&mut self, class: &str, style: T) {
        self.class_styles.insert((TypeId::of::<T>(), class.to_owned()), Box::new(style));
    }
    pub fn register_widget_style<S: Draw + Component + 'static, T: ComponentStyle<Component = S> + Debug + Send + 'static>(&mut self, widget_id: WidgetId, style: T) {
        self.widget_styles.insert((TypeId::of::<T>(), widget_id), Box::new(style));
    }
    pub fn register_class_prop_style<S: Draw + Component + 'static, T: ComponentStyle<Component = S> + Debug + Send + 'static>(&mut self, class: &str, props: PropSet, style: T) {
        self.class_style_selectors.entry((TypeId::of::<T>(), class.to_owned())).or_insert_with(LinkedHashMap::new).insert(props, Box::new(style));
    }
    pub fn register_widget_prop_style<S: Draw + Component + 'static, T: ComponentStyle<Component = S> + Debug + Send + 'static>(&mut self, widget_id: WidgetId, props: PropSet, style: T) {
        self.widget_style_selectors.entry((TypeId::of::<T>(), widget_id)).or_insert_with(LinkedHashMap::new).insert(props, Box::new(style));
    }

    pub fn get_style(&self, type_id: TypeId, class: Option<String>, widget_id: WidgetId, props: PropSet) -> DrawWrapper {
        let mut style = self.type_styles.get(&type_id).unwrap().clone();
        if let Some(class) = class {
            if let Some(class_style) = self.class_styles.get(&(type_id, class.clone())) {
                style = class_style.clone().merge(style.clone());
            }
            if let Some(selector) = self.class_style_selectors.get(&(type_id, class)) {
                style = selector.select(style, &props);
            }
        }
        if let Some(widget_style) = self.widget_styles.get(&(type_id, widget_id)) {
            style = widget_style.clone().merge(style.clone());
        }
        if let Some(selector) = self.widget_style_selectors.get(&(type_id, widget_id)) {
            style = selector.select(style, &props);
        }
        style.wrapper()
    }

    pub fn register_modifier_type_style<C: Component + WidgetModifier + 'static, T: ComponentStyle<Component = C> + Debug + Send>(&mut self, style: T) {
        self.modifier_type_styles.insert(TypeId::of::<T>(), Box::new(style));
    }

    pub fn get_modifier_style(&self, style: Box<ModifierMergeStyle>, type_id: TypeId) -> Box<ModifierMergeStyle> {
        style.merge(self.modifier_type_styles.get(&type_id).unwrap().clone())
    }
}

trait Selector {
    fn select(&self, style: Box<DrawMergeStyle>, props: &PropSet) -> Box<DrawMergeStyle>;
}
impl Selector for LinkedHashMap<PropSet, Box<DrawMergeStyle>> {
    fn select(&self, mut style: Box<DrawMergeStyle>, props: &PropSet) -> Box<DrawMergeStyle> {
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

trait DrawStyle {
    fn wrapper(self: Box<Self>) -> DrawWrapper;
}

pub trait DrawMergeStyle: Debug + Send {
    fn merge(self: Box<Self>, lower: Box<DrawMergeStyle>) -> Box<DrawMergeStyle>;
    fn wrapper(self: Box<Self>) -> DrawWrapper;
    fn box_clone(&self) -> Box<DrawMergeStyle>;
    fn as_any(&self) -> &Any;
}

impl Clone for Box<DrawMergeStyle> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}

impl <T: Draw + Component + 'static, C: ComponentStyle<Component = T> + Debug + Send> DrawMergeStyle for C {
    fn merge(self: Box<Self>, lower: Box<DrawMergeStyle>) -> Box<DrawMergeStyle> {
        let upper = self.as_any().downcast_ref::<C>().unwrap();
        let lower = lower.as_any().downcast_ref::<C>().unwrap();
        Box::new(upper.merge(lower))
    }
    fn wrapper(self: Box<Self>) -> DrawWrapper {
        DrawWrapper::new(self.component())
    }
    fn box_clone(&self) -> Box<DrawMergeStyle> {
        Box::new((*self).clone())
    }
    fn as_any(&self) -> &Any {
        self
    }
}

pub trait ModifierMergeStyle: Debug + Send {
    fn merge(self: Box<Self>, lower: Box<ModifierMergeStyle>) -> Box<ModifierMergeStyle>;
    fn comp(self: Box<Self>) -> Box<WidgetModifier>;
    fn box_clone(&self) -> Box<ModifierMergeStyle>;
    fn as_any(&self) -> &Any;
}

impl Clone for Box<ModifierMergeStyle> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}

impl <T: WidgetModifier + Component + 'static, C: ComponentStyle<Component = T> + Debug + Send> ModifierMergeStyle for C {
    fn merge(self: Box<Self>, lower: Box<ModifierMergeStyle>) -> Box<ModifierMergeStyle> {
        let upper = self.as_any().downcast_ref::<C>().unwrap();
        let lower = lower.as_any().downcast_ref::<C>().unwrap();
        Box::new(upper.merge(lower))
    }
    fn comp(self: Box<Self>) -> Box<WidgetModifier> {
        Box::new(self.component())
    }
    fn box_clone(&self) -> Box<ModifierMergeStyle> {
        Box::new((*self).clone())
    }
    fn as_any(&self) -> &Any {
        self
    }
}
