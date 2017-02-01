pub mod font;
pub mod image;

use std;
use std::sync::{Mutex, MutexGuard};

use self::font::Font;
use self::image::Texture;

/// A type-safe wrapper around a resource `Id`.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id(pub usize);

impl Id {
    /// Returns the inner `usize` from the `Id`.
    pub fn index(self) -> usize {
        self.0
    }
}

pub struct IdGen {
    id: usize,
}
impl IdGen {
    pub fn new() -> Self {
        IdGen { id: 0 }
    }
    pub fn next(&mut self) -> Id {
        let id = self.id;
        self.id = id.wrapping_add(1);
        Id(id)
    }
}

pub struct Map<T> {
    id_gen: IdGen,
    map: std::collections::HashMap<Id, T>,
}

impl<T> Map<T> {
    /// Construct the new, empty `Map`.
    pub fn new() -> Self {
        Map {
            id_gen: IdGen::new(),
            map: std::collections::HashMap::new(),
        }
    }
    /// Borrow the resource associated with the given `Id`.
    pub fn get(&self, id: Id) -> Option<&T> {
        self.map.get(&id)
    }

    /// Adds the given resource to the `Map` and returns a unique `Id` for it.
    pub fn insert(&mut self, resource: T) -> Id {
        let id = self.id_gen.next();
        self.map.insert(id, resource);
        id
    }
}

pub struct Resources {
    pub fonts: Map<Font>,
    pub images: Map<Texture>,
    pub widget_id: IdGen,
}
impl Resources {
    pub fn new() -> Self {
        Resources {
            fonts: Map::new(),
            images: Map::new(),
            widget_id: IdGen::new(),
        }
    }
    pub fn widget_id(&mut self) -> Id {
        self.widget_id.next()
    }
}
lazy_static! {
    pub static ref RES: Mutex<Resources> = Mutex::new(Resources::new());
}
pub fn resources() -> MutexGuard<'static, Resources> {
    RES.lock().unwrap()
}
