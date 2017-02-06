pub mod font;
pub mod image;

use std;
use std::sync::{Mutex, MutexGuard};
use std::fmt::Debug;

use self::font::Font;
use self::image::Texture;

pub trait Id: Copy + Clone + Debug + Hash + PartialEq + Eq + PartialOrd + Ord {
    fn new(index: usize) -> Self;
    fn index(&self) -> usize;
}

/// A type-safe wrapper around a resource `Id`.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct WidgetId(pub usize);
impl Id for WidgetId {
    fn new(index: usize) -> Self {
        WidgetId(index)
    }
    fn index(&self) -> usize {
        self.0
    }
}
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct FontId(pub usize);
impl Id for FontId {
    fn new(index: usize) -> Self {
        FontId(index)
    }
    fn index(&self) -> usize {
        self.0
    }
}
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ImageId(pub usize);
impl Id for ImageId {
    fn new(index: usize) -> Self {
        ImageId(index)
    }
    fn index(&self) -> usize {
        self.0
    }
}
use std::marker::PhantomData;
use std::hash::Hash;
pub struct IdGen<I> {
    id: usize,
    phantom: PhantomData<I>
}
impl<I: Id> IdGen<I> {
    pub fn new() -> Self {
        IdGen { id: 0, phantom: PhantomData }
    }
    pub fn next(&mut self) -> I {
        let id = self.id;
        self.id = id.wrapping_add(1);
        Id::new(id)
    }
}

pub struct Map<I, T> {
    id_gen: IdGen<I>,
    map: std::collections::HashMap<I, T>,
}

impl<I: Id + Eq + Hash, T> Map<I, T> {
    /// Construct the new, empty `Map`.
    pub fn new() -> Self {
        Map {
            id_gen: IdGen::new(),
            map: std::collections::HashMap::new(),
        }
    }
    /// Borrow the resource associated with the given `Id`.
    pub fn get(&self, id: I) -> Option<&T> {
        self.map.get(&id)
    }

    /// Adds the given resource to the `Map` and returns a unique `Id` for it.
    pub fn insert(&mut self, resource: T) -> I {
        let id = self.id_gen.next();
        self.map.insert(id, resource);
        id
    }
}

pub struct Resources {
    pub fonts: Map<FontId, Font>,
    pub images: Map<ImageId, Texture>,
    pub widget_id: IdGen<WidgetId>,
}
impl Resources {
    pub fn new() -> Self {
        Resources {
            fonts: Map::new(),
            images: Map::new(),
            widget_id: IdGen::new(),
        }
    }
    pub fn widget_id(&mut self) -> WidgetId {
        self.widget_id.next()
    }
}
lazy_static! {
    pub static ref RES: Mutex<Resources> = Mutex::new(Resources::new());
}
pub fn resources() -> MutexGuard<'static, Resources> {
    RES.lock().unwrap()
}
