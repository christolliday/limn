use std::any::{Any, TypeId};
use std::collections::HashMap;

use resources;
use widget::WidgetBuilder;


pub struct Theme {
    styles: HashMap<TypeId, Box<Any + Send>>,
    style_classes: HashMap<(TypeId, String), Box<Any + Send>>,
}

impl Theme {
    pub fn new() -> Self {
        Theme {
            styles: HashMap::new(),
            style_classes: HashMap::new(),
        }
    }

    pub fn register_style<T: ComponentStyle + Send + 'static>(&mut self, style: T) {
        self.styles.insert(TypeId::of::<T>(), Box::new(style));
    }
    pub fn register_style_class<T: ComponentStyle + Send + 'static>(&mut self, class: &str, style: T) {
        self.style_classes.insert((TypeId::of::<T>(), class.to_owned()), Box::new(style));
    }
    pub fn get_style<T: ComponentStyle + 'static>(&self, mut style: T, class: Option<String>) -> T {
        let type_id = TypeId::of::<T>();
        if let Some(type_class) = self.styles.get(&type_id) {
            let type_class = type_class.downcast_ref::<T>().unwrap();
            style = style.merge(type_class);
        }
        if let Some(class) = class {
            if let Some(class) = self.style_classes.get(&(type_id, class)) {
                let class = class.downcast_ref::<T>().unwrap();
                style = style.merge(class);
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
    fn resolve(self, class: Option<String>) -> Self::Component {
        let res = resources::resources();
        let style = self.clone();
        res.theme.get_style(style, class).component()
    }
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
    fn resolve(self, _: Option<String>) -> Self::Component {
        self
    }
}

pub trait MergeStyle {
    fn merge(&self, lower: &Self) -> Self;
}
