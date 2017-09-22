use std::ops::Deref;

use linked_hash_map::LinkedHashMap;

use widget::PropSet;
use widget::draw::Draw;

#[derive(Clone, Debug)]
pub enum Value<T: Clone> {
    Single(T),
    Selector(Selector<T>),
}
impl<T: Clone> From<T> for Value<T> {
    fn from(val: T) -> Self {
        Value::Single(val)
    }
}
impl<T: Clone> From<Selector<T>> for Value<T> {
    fn from(val: Selector<T>) -> Self {
        Value::Selector(val)
    }
}

#[derive(Clone, Debug)]
pub struct Selector<T> {
    // uses a linked hashmap to allow ordering of matches by priority
    // the first ordered propset that is a subset of the widgets props will be matched
    pub matcher: LinkedHashMap<PropSet, T>,
    pub default: T,
}
impl<T> Selector<T> {
    pub fn new(default: T) -> Self {
        Selector {
            matcher: LinkedHashMap::new(),
            default: default,
        }
    }
    pub fn insert<P: Deref<Target=PropSet>>(&mut self, props: &P, value: T) {
        self.matcher.insert(props.deref().clone(), value);
    }
}

impl<T: Clone> Value<T> {
    pub fn get(&self, props: &PropSet) -> T {
        let val = match *self {
            Value::Selector::<T>(ref sel) => {
                let &Selector { ref matcher, ref default } = sel;
                if matcher.contains_key(props) {
                    matcher.get(props).unwrap()
                } else {
                    matcher.iter().find(|&(matcher_props, _)| {
                        matcher_props.is_subset(props)
                    }).map(|(_, val)| val).unwrap_or(default)
                }
            },
            Value::Single(ref val) => val
        };
        val.clone()
    }
    pub fn default(&self) -> T {
        let val = match *self {
            Value::Single::<T>(ref val) => val,
            Value::Selector::<T>(ref sel) => &sel.default,
        };
        val.clone()
    }
}

impl<D: Draw, S: Style<D>> Style<D> for Vec<S> {
    fn apply(&self, draw_state: &mut D, props: &PropSet) {
        for field in self.iter() {
            field.apply(draw_state, props);
        }
    }
}

pub trait Style<D> {
    fn apply(&self, state: &mut D, props: &PropSet);
}

pub fn apply_style<D, S: Style<D>>(state: &mut D, style: &[S], props: &PropSet) {
    for field in style.iter() {
        field.apply(state, props);
    }
}

#[macro_export]
macro_rules! selector {
    ($default:expr, $($props:ident: $val:expr),*) => {
        {
            use $crate::widget::style::Selector;
            let mut selector = Selector::new($default);
            $(
                selector.insert(&$props, $val);
            )*
            selector
        }
    }
}
#[macro_export]
macro_rules! style {
    (parent: $parent:expr, $($type:path: $val:expr,)*) => {
        style!(parent: $parent, $($type:path: $val:expr),*);
    };
    (parent: $parent:expr, $($type:path: $val:expr),*) => {
        {
            use $crate::widget::style::Value;
            let mut style = $parent.clone();
            $(
                style.push($type(Value::from($val)));
            )*
            style
        }
    };
    ($($type:path: $val:expr,)*) => {
        style!($($type:path: $val:expr),*);
    };
    ($($type:path: $val:expr),*) => {
        {
            use $crate::widget::style::Value;
            vec![
                $(
                    $type(Value::from($val)),
                )*
            ]
        }
    };
}

pub struct StyleUpdated;
