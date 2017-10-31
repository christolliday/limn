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

    pub fn register_style<T: Component + Send + 'static>(&mut self, style: T) {
        self.styles.insert(TypeId::of::<T>(), Box::new(style));
    }
    pub fn register_style_class<T: Component + Send + 'static>(&mut self, class: &str, style: T) {
        self.style_classes.insert((TypeId::of::<T>(), class.to_owned()), Box::new(style));
    }
    pub fn get_style<T: Component + Default + 'static>(&self, mut style: T, class: Option<String>) -> T {
        let type_id = TypeId::of::<T>();
        if let Some(class) = class {
            if let Some(class) = self.style_classes.get(&(type_id, class)) {
                let class = class.downcast_ref::<T>().unwrap();
                style = style.merge(class);
            }
        }
        if let Some(type_class) = self.styles.get(&type_id) {
            let type_class = type_class.downcast_ref::<T>().unwrap();
            style = style.merge(type_class);
        }
        let default = T::default();
        style.merge(&default)
    }
}

pub trait ComponentValues: ::std::fmt::Debug {
    fn apply(&self, widget: &mut WidgetBuilder);
}

pub trait Component: Default + Clone + 'static {
    type Values: ComponentValues;
    fn name() -> String;
    fn merge(&self, other: &Self) -> Self;
    fn apply(&self, widget: &mut WidgetBuilder) {
        let style = {
            let res = resources::resources();
            let class = widget.style_class();
            let style = self.clone();
            res.theme.get_style(style, class)
        }.to_values();
        style.apply(widget);
    }
    fn to_values(self) -> Self::Values;
}

pub trait MergeStyle {
    fn merge(&self, lower: &Self) -> Self;
}

use draw::text::TextStyle;

impl MergeStyle for Option<Option<Vec<TextStyle>>> {
    fn merge(&self, lower: &Self) -> Self {
        if self.is_none() {
            return lower.clone();
        } else if lower.is_none() || lower.as_ref().unwrap().is_none() {
            return self.clone();
        }
        if let (&Some(ref this), &Some(ref lower)) = (self, lower) {
            if let (&Some(ref this), &Some(ref lower)) = (this, lower) {
                let mut vec = Vec::new();
                vec.append(&mut this.clone());
                vec.append(&mut lower.clone());
                return Some(Some(vec));
            } else {
                return Some(None);
            }
        }
        unreachable!();
    }
}

impl MergeStyle for Option<Vec<TextStyle>> {
    fn merge(&self, lower: &Self) -> Self {
        if self.is_none() {
            return lower.clone();
        } else if lower.is_none() {
            return self.clone();
        }
        if let (&Some(ref this), &Some(ref lower)) = (self, lower) {
            let mut vec = Vec::new();
            vec.append(&mut this.clone());
            vec.append(&mut lower.clone());
            return Some(vec);
        }
        unreachable!();
    }
}
