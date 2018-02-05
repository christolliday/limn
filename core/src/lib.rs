//! Limn is a cross platform, event driven, component based GUI library.

// uncomment to debug macros
//#![feature(trace_macros)]
//trace_macros!(true);

#![cfg_attr(feature="nightly", feature(core_intrinsics))]
#![cfg_attr(feature="nightly", feature(get_type_id))]

extern crate limn_layout;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate maplit;
#[macro_use]
extern crate log;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate mopa;

pub extern crate limn_text_layout as text_layout;
pub extern crate cassowary;
pub extern crate rusttype;
pub extern crate glutin;
pub extern crate webrender;
extern crate euclid;
extern crate linked_hash_map;
extern crate stable_bst;
extern crate gleam;
extern crate app_units;
extern crate image;
extern crate font_loader;

#[macro_use]
pub mod style;
#[macro_use]
pub mod event;
/// Module for `Widget` and callback handlers
#[macro_use]
pub mod widget;
/// Module for layout / resizing handlers and layout solving
#[macro_use]
pub mod layout;

pub mod app;
pub mod ui;
pub mod geometry;
/// Font, image and texture resources
pub mod resources;
pub mod color;
pub mod input;
pub mod prelude;
pub mod render;
/// Wrapper around `glutin::Window`
pub mod window;

#[cfg(not(feature="nightly"))]
fn type_name<T>() -> &'static str {
    "Type unavailable, use nightly"
}

#[cfg(feature="nightly")]
fn type_name<T>() -> &'static str {
    unsafe { std::intrinsics::type_name::<T>() }
}
