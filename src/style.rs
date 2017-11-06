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
    fn merge(&self, other: &Self) -> Self {
        self.clone()
    }
    fn component(self) -> Self::Component {
        self
    }
    fn resolve(self, class: Option<String>) -> Self::Component {
        self
    }
}

/* impl <T: ComponentStyle> Component for T {
    fn name() -> String {
        T::Component::name()
    }
} */

/* impl <T: ComponentStyle<Component = S>, S: WidgetModifier + Component> WidgetModifier for T {
    fn apply(&self, widget: &mut WidgetBuilder) {
        let style = {
            let res = resources::resources();
            let class = widget.style_class();
            let style = self.clone();
            res.theme.get_style(style, class)
        }.component();
        style.apply(widget);
    }
} */

pub trait MergeStyle {
    fn merge(&self, lower: &Self) -> Self;
}

/* use draw::text::TextStyle;

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
} */
