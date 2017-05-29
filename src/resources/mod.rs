pub mod font;
pub mod image;
#[macro_use]
pub mod id;

use std::sync::{Mutex, MutexGuard};
use std::collections::HashMap;

use self::id::{Id, IdGen};

use self::font::Font;
use self::image::Texture;

lazy_static! {
    static ref RES: Mutex<Resources> = Mutex::new(Resources::new());
}

// Allow global access to Resources
pub fn resources() -> MutexGuard<'static, Resources> {
    RES.lock().unwrap()
}

named_id!(WidgetId);
named_id!(FontId);
named_id!(ImageId);

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
