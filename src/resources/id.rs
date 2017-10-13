use std::fmt::Debug;
use std::marker::PhantomData;
use std::hash::Hash;
use std::default::Default;

pub trait Id: Copy + Clone + Debug + Hash + PartialEq + Eq + PartialOrd + Ord {
    fn new(index: usize) -> Self;
}

/// Create a new simple id type, wrapper around a usize that can be created via `IdGen`
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

/// Generates named Ids, wrappers around increasing usize values.
/// For Ids to be unique, just needs to be one `IdGen` per Id type.
pub struct IdGen<I> {
    id: usize,
    phantom: PhantomData<I>,
}

impl<I: Id> IdGen<I> {
    fn default() -> Self {
        IdGen {
            id: 0,
            phantom: PhantomData,
        }
    }
}

impl<I: Id> IdGen<I> {

    pub fn new() -> Self {
        Self::default()
    }

    /// Not to be confused with `std::iterator::Iterator::next()`!
    /// This function simply increases the Id by 1 while keeping the type
    pub fn next_id(&mut self) -> I {
        let id = self.id;
        self.id = id.wrapping_add(1);
        Id::new(id)
    }
}

named_id!(WidgetId);
