use std::ops::Deref;

use linked_hash_map::LinkedHashMap;

use widget::PropSet;
use widget::drawable::Drawable;

#[derive(Clone, Debug)]
pub enum Value<T>
    where T: Clone
{
    Single(T),
    // uses a linked hashmap to allow ordering of matches by priority
    // the first ordered propset that is a subset of the widgets props will be matched
    Selector(Selector<T>),
}

#[derive(Clone, Debug)]
pub struct Selector<T> {
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

impl<T> Value<T>
    where T: Clone
{
    pub fn from_props(&self, props: &PropSet) -> T {
        let val = match *self {
            Value::Selector::<T>(ref sel) => {
                let &Selector { ref matcher, ref default } = sel;
                if matcher.contains_key(&props) {
                    matcher.get(&props).unwrap()
                } else {
                    matcher.iter().find(|&(matcher_props, _)| {
                        matcher_props.is_subset(&props)
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

pub trait Style<D: Drawable> {
    fn apply(&self, drawable: &mut D, props: &PropSet);
}
impl<D: Drawable, S: StyleField<D>> Style<D> for Vec<S> {
    fn apply(&self, drawable: &mut D, props: &PropSet) {
        for field in self.iter() {
            field.apply(drawable, props);
        }
    }
}

pub trait StyleField<D> {
    fn apply(&self, state: &mut D, props: &PropSet);
}

pub fn apply_style<D, S: StyleField<D>>(state: &mut D, style: &Vec<S>, props: &PropSet) {
    for field in style.iter() {
        field.apply(state, props);
    }
}
