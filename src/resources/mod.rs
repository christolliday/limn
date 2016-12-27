pub mod font;
pub mod image;

/// The `font::Id` and `font::Map` types.
use std;

/// A type-safe wrapper around the `FontId`.
///
/// This is used as both:
///
/// - The key for the `font::Map`'s inner `HashMap`.
/// - The `font_id` field for the rusttype::gpu_cache::Cache.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id(usize);

/// A collection of mappings from `font::Id`s to `rusttype::Font`s.
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
    /// Borrow the `rusttype::Font` associated with the given `font::Id`.
    pub fn get(&self, id: Id) -> Option<&T> {
        self.map.get(&id)
    }

    /// Adds the given `rusttype::Font` to the `Map` and returns a unique `Id` for it.
    pub fn insert(&mut self, resource: T) -> Id {
        let index = self.next_index;
        self.next_index = index.wrapping_add(1);
        let id = Id(index);
        self.map.insert(id, resource);
        id
    }
}
