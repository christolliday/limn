use std::any::TypeId;
use std::fmt;
use std::fmt::Debug;

use linked_hash_map::LinkedHashMap;
use style::DrawComponentStyle;
use style::ComponentStyle;
use style::Component;
use widget::property::PropSet;
use widget::draw::Draw;
use resources::resources;

#[derive(Debug)]
pub struct DrawStyle {
    pub style: Option<Box<DrawComponentStyle>>,
    pub selector: Option<LinkedHashMap<PropSet, Box<DrawComponentStyle>>>,
    pub class: Option<String>,
    pub type_id: TypeId,
}

impl DrawStyle {
    pub fn new<D: Draw + Component + 'static, T: ComponentStyle<Component = D> + Debug + Send + 'static>(style: T) -> Self {
        DrawStyle {
            style: Some(Box::new(style)),
            selector: None,
            class: None,
            type_id: TypeId::of::<T>(),
        }
    }
    pub fn from_class<T: 'static>(class: &str) -> Self {
        DrawStyle {
            style: None,
            selector: None,
            class: Some(class.to_owned()),
            type_id: TypeId::of::<T>(),
        }
    }
    pub fn prop_style<D: Draw + Component + 'static, T: ComponentStyle<Component = D> + Debug + Send + 'static>(&mut self, props: PropSet, style: T) {
        self.selector.get_or_insert_with(LinkedHashMap::default).insert(props, Box::new(style));
    }
    pub fn set_class(&mut self, class: &str) {
        self.class = Some(class.to_owned());
    }
    pub fn resolve(&self, props: PropSet) -> Box<DrawComponentStyle> {
        let res = resources();
        res.theme.get_style(self, props)
    }
    pub fn merge(&mut self, other: DrawStyle) {
        if let Some(other_style) = other.style {
            if let Some(style) = self.style.clone() {
                self.style = Some(other_style.merge(style));
            } else {
                self.style = Some(other_style);
            }
        }
        if let Some(mut other_selector) = other.selector {
            if self.selector.is_none() {
                self.selector = Some(LinkedHashMap::new());
            }
            let selector = self.selector.as_mut().unwrap();
            for entry in other_selector.entries() {
                if selector.contains_key(entry.key()) {
                    let style = selector.get(entry.key()).unwrap().clone();
                    selector.insert(entry.key().clone(), style.merge(entry.get().clone()));
                } else {
                    selector.insert(entry.key().clone(), entry.get().clone());
                }
            }
        }
        if let Some(other_class) = other.class {
            self.class = Some(other_class);
        }
    }
}

impl <D: Draw + Component + 'static, T: ComponentStyle<Component = D> + Send + Debug + 'static> From<T> for DrawStyle {
    fn from(w: T) -> DrawStyle {
        DrawStyle::new(w)
    }
}

#[derive(Default)]
pub struct DrawState {
    pub style: Option<DrawStyle>,
    pub state: Option<Box<Draw>>,

    style_updated: bool,
}

impl Debug for DrawState {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}

impl DrawState {

    pub fn style_updated(&mut self) {
        self.style_updated = true;
    }

    pub fn update(&mut self, props: PropSet) {
        if let Some(style) = self.style.as_ref() {
            self.state = Some(style.resolve(props).wrapper());
            self.style_updated = false;
        }
    }

    pub fn needs_update(&self) -> bool {
        self.style_updated
    }
    pub fn set_draw_state<T: Draw + Component + 'static>(&mut self, draw_state: T) -> &mut Self {
        self.state = Some(Box::new(draw_state));
        self.style_updated();
        self
    }
    pub fn set_draw_style(&mut self, new_style: DrawStyle) {
        if let Some(ref mut style) = self.style {
            style.merge(new_style);
        } else {
            self.style = Some(new_style);
        }
        self.style_updated();
    }
    pub fn get_state<T: Draw>(&self) -> &T {
        self.state.as_ref().unwrap().downcast_ref::<T>().unwrap()
    }
}
