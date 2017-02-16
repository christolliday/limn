pub mod font;
pub mod image;

use std::sync::{Mutex, MutexGuard};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::hash::Hash;
use std::collections::HashMap;

use self::font::Font;
use self::image::Texture;

lazy_static! {
    pub static ref RES: Mutex<Resources> = Mutex::new(Resources::new());
}
pub fn resources() -> MutexGuard<'static, Resources> {
    RES.lock().unwrap()
}

pub trait Id: Copy + Clone + Debug + Hash + PartialEq + Eq + PartialOrd + Ord {
    fn new(index: usize) -> Self;
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct WidgetId(pub usize);
impl Id for WidgetId {
    fn new(index: usize) -> Self {
        WidgetId(index)
    }
}
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct FontId(pub usize);
impl Id for FontId {
    fn new(index: usize) -> Self {
        FontId(index)
    }
}
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ImageId(pub usize);
impl Id for ImageId {
    fn new(index: usize) -> Self {
        ImageId(index)
    }
}

pub struct IdGen<I> {
    id: usize,
    phantom: PhantomData<I>,
}
impl<I: Id> IdGen<I> {
    pub fn new() -> Self {
        IdGen {
            id: 0,
            phantom: PhantomData,
        }
    }
    pub fn next(&mut self) -> I {
        let id = self.id;
        self.id = id.wrapping_add(1);
        Id::new(id)
    }
}

pub struct Map<I, T> {
    id_gen: IdGen<I>,
    map: HashMap<I, T>,
}

impl<I: Id, T> Map<I, T> {
    /// Construct the new, empty `Map`.
    pub fn new() -> Self {
        Map {
            id_gen: IdGen::new(),
            map: HashMap::new(),
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
