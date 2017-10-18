//! Limn is a cross platform, event driven, component based GUI library.

#![cfg_attr(feature="nightly", feature(core_intrinsics))]

#[macro_use]
pub extern crate limn_layout;
pub extern crate text_layout;
pub extern crate glutin;
pub extern crate gleam;
pub extern crate image;
pub extern crate rusttype;
pub extern crate webrender;
pub extern crate app_units;

#[cfg(feature = "font_loader")]
pub extern crate font_loader;

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate maplit;
#[macro_use]
extern crate downcast_rs;
#[macro_use]
extern crate log;
extern crate linked_hash_map;
extern crate stable_bst;
extern crate euclid;
extern crate cassowary;

#[macro_use]
pub mod event;
#[macro_use]
pub mod widget;
#[macro_use]
pub mod layout;
pub mod app;
pub mod widgets;
pub mod draw;
pub mod ui;
pub mod geometry;
pub mod resources;
pub mod color;
pub mod input;
pub mod prelude;
pub mod render;
pub mod window;
pub mod errors;

#[cfg(not(feature="nightly"))]
fn type_name<T>() -> &'static str {
    "Type unavailable, use nightly"
}

#[cfg(feature="nightly")]
fn type_name<T>() -> &'static str {
    unsafe { std::intrinsics::type_name::<T>() }
}
