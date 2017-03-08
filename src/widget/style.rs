use linked_hash_map::LinkedHashMap;

use widget::PropSet;
use widget::drawable::Drawable;

#[derive(Clone, Debug)]
pub enum Value<T>
    where T: Clone
{
    Single(T),
    Selector((LinkedHashMap<PropSet, T>, T)),
}

impl<T> Value<T>
    where T: Clone
{
    pub fn from_props(&self, props: &PropSet) -> T {
        match *self {
            Value::Selector::<T>((ref sel, _)) => {
                if sel.contains_key(&props) {
                    return sel.get(&props).unwrap().clone();
                } else {
                    for (style_props, style_val) in sel.iter() {
                        // props matches all in style props
                        if style_props.is_subset(&props) {
                            return style_val.clone();
                        }
                    }
                }
            }
            _ => (),
        }
        self.default()
    }
    pub fn default(&self) -> T {
        match *self {
            Value::Single::<T>(ref val) => val.clone(),
            Value::Selector::<T>((_, ref def)) => def.clone(),
        }
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
