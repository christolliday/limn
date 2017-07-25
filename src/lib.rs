#![cfg_attr(feature="nightly", feature(core_intrinsics))]

extern crate backend;
extern crate text_layout;
#[macro_use]
extern crate limn_layout;
extern crate graphics;
extern crate cassowary;
extern crate rusttype;
extern crate gfx_device_gl;
extern crate gfx_graphics;
extern crate glutin;
#[macro_use]
extern crate lazy_static;
extern crate linked_hash_map;
extern crate stable_bst;
#[macro_use]
extern crate maplit;
#[macro_use]
extern crate downcast_rs;
extern crate euclid;
#[macro_use]
extern crate log;
extern crate multi_mut;

#[macro_use]
pub mod event;
pub mod app;
#[macro_use]
pub mod widget;
#[macro_use]
pub mod layout;
pub mod widgets;
pub mod drawable;
pub mod ui;
pub mod util;
pub mod resources;
pub mod color;
pub mod input;
pub mod prelude;

#[cfg(not(feature="nightly"))]
fn type_name<T>() -> &'static str {
    "Type unavailable, use nightly"
}

#[cfg(feature="nightly")]
fn type_name<T>() -> &'static str {
    unsafe { std::intrinsics::type_name::<T>() }
}
