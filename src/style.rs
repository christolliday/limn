/*!
Contains types relevant to declaring styleable components, and the theming engine which allows styleable components to inherit values from an application theme.

Styleable components are arbitrary structs that can inherit values from a theme by defining a style struct that can be converted into the original struct
by implementing the trait `ComponentStyle`.

Currently this can be used for structs that implement `Draw` and `WidgetModifier` and so can be used to style both drawable types,
and complex widgets.

The macro `component_style!` can be used to simplify defining styleable types:

```
# #[macro_use] extern crate limn; fn main() {
# use limn::draw::rect::RectStyle;
# use limn::draw::text::TextStyle;
component_style!{pub struct Button<name="button", style=ButtonStyle> {
    rect: RectStyle = RectStyle::default(),
    text: Option<TextStyle> = None,
}}
# }
```

This declares two structs:

```
# use limn::draw::rect::RectStyle;
# use limn::draw::text::TextStyle;
struct Button {
    rect: RectStyle,
    text: Option<TextStyle>,
}
struct ButtonStyle {
    rect: Option<RectStyle>,
    text: Option<Option<TextStyle>>,
}
```

and implements various traits on them, but most importantly:

`impl ComponentStyle<Component = Button> for ButtonStyle`

If you then implement `WidgetModifier` for `Button`, you can define how to initialize a button using the `rect` and `text` fields.
Then by passing a `ButtonStyle` in place of a `Button` to initialize a widget, you can specify only the fields you want to be
specific to that widget, with the remaining fields inherited from the theme.

The theme will prioritize which values to use by (currently) simple specificity rules:

-The specific style passed to a widget
-Styles registered in the theme for named style classes that can be applied to widgets, eg. "alert_button"
-The base style for the type, ie. `ButtonStyle`, registered in the theme
-In values are found no where else, the default values specified in `component_style!`, in this example, `RectStyle::default()` and `None`
*/

use std::fmt::{self, Debug};
use std::any::{Any, TypeId};
use std::collections::HashMap;

use widget::Widget;
use widget::draw::Draw;
use widget::property::PropSet;

use resources::resources;

use linked_hash_map::LinkedHashMap;

pub struct Theme {
    type_styles: HashMap<TypeId, Box<DrawComponentStyle>>,
    class_styles: HashMap<(TypeId, String), Box<DrawComponentStyle>>,
    class_style_selectors: HashMap<(TypeId, String), LinkedHashMap<PropSet, Box<DrawComponentStyle>>>,
    modifier_type_styles: HashMap<TypeId, Box<ModifierComponentStyle>>,
    modifier_class_styles: HashMap<(TypeId, String), Box<ModifierComponentStyle>>,
}

impl Theme {
    pub fn new() -> Self {
        Theme {
            type_styles: HashMap::new(),
            class_styles: HashMap::new(),
            class_style_selectors: HashMap::new(),
            modifier_type_styles: HashMap::new(),
            modifier_class_styles: HashMap::new(),
        }
    }

    pub fn register_type_style<S: Draw + Component + 'static, T: ComponentStyle<Component = S> + Debug + Send + 'static>(&mut self, style: T) {
        self.type_styles.insert(TypeId::of::<T>(), Box::new(style));
    }
    pub fn register_class_style<S: Draw + Component + 'static, T: ComponentStyle<Component = S> + Debug + Send + 'static>(&mut self, class: &str, style: T) {
        self.class_styles.insert((TypeId::of::<T>(), class.to_owned()), Box::new(style));
    }
    pub fn register_class_prop_style<S: Draw + Component + 'static, T: ComponentStyle<Component = S> + Debug + Send + 'static>(&mut self, class: &str, props: PropSet, style: T) {
        self.class_style_selectors.entry((TypeId::of::<T>(), class.to_owned())).or_insert_with(LinkedHashMap::new).insert(props, Box::new(style));
    }

    pub fn get_style(&self, widget_style: &DrawStyle, props: PropSet) -> Box<DrawComponentStyle> {
        let type_id = widget_style.type_id;
        let mut style = self.type_styles.get(&type_id).expect("Missing default style for type").clone();
        if let Some(ref class) = widget_style.class {
            if let Some(class_style) = self.class_styles.get(&(type_id, class.clone())) {
                style = class_style.clone().merge(style.clone());
            }
            if let Some(selector) = self.class_style_selectors.get(&(type_id, class.clone())) {
                style = selector.select(style, &props);
            }
        }
        if let Some(widget_style) = widget_style.style.as_ref() {
            style = widget_style.clone().merge(style.clone());
        }
        if let Some(ref selector) = widget_style.selector {
            style = selector.select(style, &props);
        }
        style
    }

    pub fn register_modifier_type_style<C: Component + WidgetModifier + 'static, T: ComponentStyle<Component = C> + Debug + Send>(&mut self, style: T) {
        self.modifier_type_styles.insert(TypeId::of::<T>(), Box::new(style));
    }

    pub fn register_modifier_class_style<C: Component + WidgetModifier + 'static, T: ComponentStyle<Component = C> + Debug + Send>(&mut self, class: &str, style: T) {
        self.modifier_class_styles.insert((TypeId::of::<T>(), String::from(class)), Box::new(style));
    }

    pub fn get_modifier_style(&self, style: Box<ModifierComponentStyle>, type_id: TypeId, class: Option<String>) -> Box<ModifierComponentStyle> {
        let style = if let Some(type_style) = self.modifier_type_styles.get(&type_id) {
            style.merge(type_style.clone())
        } else {
            style
        };
        if let Some(class_style) = class.and_then(|class| self.modifier_class_styles.get(&(type_id, class))) {
            style.merge(class_style.clone())
        } else {
            style
        }
    }
}

trait Selector {
    fn select(&self, style: Box<DrawComponentStyle>, props: &PropSet) -> Box<DrawComponentStyle>;
}
impl Selector for LinkedHashMap<PropSet, Box<DrawComponentStyle>> {
    fn select(&self, mut style: Box<DrawComponentStyle>, props: &PropSet) -> Box<DrawComponentStyle> {
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
    fn apply(&self, widget: &mut Widget);
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

pub trait DrawComponentStyle: Debug + Send {
    fn merge(self: Box<Self>, lower: Box<DrawComponentStyle>) -> Box<DrawComponentStyle>;
    fn wrapper(self: Box<Self>) -> Box<Draw>;
    fn box_clone(&self) -> Box<DrawComponentStyle>;
    fn as_any(&self) -> &Any;
}

impl Clone for Box<DrawComponentStyle> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}

impl <T: Draw + Component + 'static, C: ComponentStyle<Component = T> + Debug + Send> DrawComponentStyle for C {
    fn merge(self: Box<Self>, lower: Box<DrawComponentStyle>) -> Box<DrawComponentStyle> {
        let upper = self.as_any().downcast_ref::<C>().unwrap();
        let lower = lower.as_any().downcast_ref::<C>().unwrap();
        Box::new(upper.merge(lower))
    }
    fn wrapper(self: Box<Self>) -> Box<Draw> {
        Box::new(self.component())
    }
    fn box_clone(&self) -> Box<DrawComponentStyle> {
        Box::new((*self).clone())
    }
    fn as_any(&self) -> &Any {
        self
    }
}

pub trait ModifierComponentStyle: Debug + Send {
    fn merge(self: Box<Self>, lower: Box<ModifierComponentStyle>) -> Box<ModifierComponentStyle>;
    fn comp(self: Box<Self>) -> Box<WidgetModifier>;
    fn box_clone(&self) -> Box<ModifierComponentStyle>;
    fn as_any(&self) -> &Any;
}

impl Clone for Box<ModifierComponentStyle> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}

impl <T: WidgetModifier + Component + 'static, C: ComponentStyle<Component = T> + Debug + Send> ModifierComponentStyle for C {
    fn merge(self: Box<Self>, lower: Box<ModifierComponentStyle>) -> Box<ModifierComponentStyle> {
        let upper = self.as_any().downcast_ref::<C>().unwrap();
        let lower = lower.as_any().downcast_ref::<C>().unwrap();
        Box::new(upper.merge(lower))
    }
    fn comp(self: Box<Self>) -> Box<WidgetModifier> {
        Box::new(self.component())
    }
    fn box_clone(&self) -> Box<ModifierComponentStyle> {
        Box::new((*self).clone())
    }
    fn as_any(&self) -> &Any {
        self
    }
}

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


#[macro_export]
macro_rules! component_style {
    ( pub struct $component:ident <name=$name:expr, style=$style:ident> { $ ( $field:ident : $field_type:ty = $default:expr, ) * } ) => {
        #[allow(missing_copy_implementations)]
        #[derive(Clone, Debug)]
        pub struct $component {
            $(
                pub $field: $field_type,
            )*
        }
        impl Default for $component {
            fn default() -> Self {
                $component {
                    $(
                        $field: $default,
                    )*
                }
            }
        }
        impl $crate::style::Component for $component {
            fn name() -> String {
                $name.to_owned()
            }
        }
        #[allow(missing_copy_implementations)]
        #[derive(Clone, Debug, Default)]
        pub struct $style {
            $(
                pub $field: Option<$field_type>,
            )*
        }
        impl $crate::style::ComponentStyle for $style {
            type Component = $component;
            fn merge(&self, other: &Self) -> Self {
                $style {
                    $(
                        $field: self.$field.as_ref().or(other.$field.as_ref()).cloned(),
                    )*
                }
            }
            fn component(self) -> Self::Component {
                $component {
                    $(
                        $field: self.$field.unwrap_or($default),
                    )*
                }
            }
        }
    };
}

#[macro_export]
macro_rules! style {
    ( $style_type:ident { $( $field:ident: $value:expr ) , * } ) => {
        $style_type {
            $(
                $field: Some($value),
            )*
            ..$style_type::default()
        }
    };
    ( $style_type:ident { $( $field:ident: $value:expr, ) * } ) => {
        style!($style_type { $($field: $value),* })
    };
}
