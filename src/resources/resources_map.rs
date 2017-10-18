use resources::id::{IdGen, Id};
use std::collections::HashMap;

/// Map for a given `Id` and resource type.
pub struct Map<I, T> {
    id_gen: IdGen<I>,
    map: HashMap<I, T>,
}

impl<I: Id, T> Map<I, T> {

    /// Creates a new Map
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

    /// Borrow the resource associated with the given `Id` mutably.
    pub fn get_mut(&self, id: I) -> Option<&mut T> {
        self.map.get_mut(&id)
    }

    /// Adds the given resource to the `Map` and returns a unique `Id` for it.
    pub fn insert(&mut self, resource: T) -> I {
        let id = self.id_gen.next();
        self.map.insert(id, resource);
        id
    }
}
