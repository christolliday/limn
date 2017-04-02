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
    static ref RES: Mutex<Resources> = Mutex::new(Resources::new());
}

// Allow global access to Resources
pub fn resources() -> MutexGuard<'static, Resources> {
    RES.lock().unwrap()
}

pub trait Id: Copy + Clone + Debug + Hash + PartialEq + Eq + PartialOrd + Ord {
    fn new(index: usize) -> Self;
}

/// Create a new simple id type, wrapper around a usize that can be created via IdGen
#[macro_export]
macro_rules! named_id {
    ($name:ident) => {
        #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $name(pub usize);
        impl Id for $name {
            fn new(index: usize) -> Self {
                $name(index)
            }
        }
    }
}

named_id!(WidgetId);
named_id!(FontId);
named_id!(ImageId);

/// Generates named Ids, wrappers around increasing usize values.
/// For Ids to be unique, just needs to be one IdGen per Id type.
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

/// Map for a given `Id` and resource type.
pub struct Map<I, T> {
    id_gen: IdGen<I>,
    map: HashMap<I, T>,
}

impl<I: Id, T> Map<I, T> {
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
