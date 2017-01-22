pub mod font;
pub mod image;

use std;

/// A type-safe wrapper around a resource `Id`.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id(pub usize);

pub struct Map<T> {
    next_index: usize,
    map: std::collections::HashMap<Id, T>,
}

impl Id {
    /// Returns the inner `usize` from the `Id`.
    pub fn index(self) -> usize {
        self.0
    }
}

impl<T> Map<T> {
    /// Construct the new, empty `Map`.
    pub fn new() -> Self {
        Map {
            next_index: 0,
            map: std::collections::HashMap::new(),
        }
    }
    /// Borrow the resource associated with the given `Id`.
    pub fn get(&self, id: Id) -> Option<&T> {
        self.map.get(&id)
    }

    /// Adds the given resource to the `Map` and returns a unique `Id` for it.
    pub fn insert(&mut self, resource: T) -> Id {
        let index = self.next_index;
        self.next_index = index.wrapping_add(1);
        let id = Id(index);
        self.map.insert(id, resource);
        id
    }
}
