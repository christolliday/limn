//! Limn is a cross platform, event driven, component based GUI library.

#![cfg_attr(feature="nightly", feature(core_intrinsics))]

#[macro_use]
extern crate limn_layout;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate maplit;
#[macro_use]
extern crate downcast_rs;
#[macro_use]
extern crate log;

extern crate text_layout;
extern crate cassowary;
extern crate rusttype;
extern crate glutin;
extern crate linked_hash_map;
extern crate stable_bst;
extern crate euclid;
extern crate webrender;
extern crate gleam;
extern crate app_units;
extern crate image;

/// Mouse & keyboard handling / delegating functions
#[macro_use]
pub mod event;
/// Module for `WidgetBuilder` and callback handlers
#[macro_use]
pub mod widget;
/// Module for layout / resizing handlers and layout solving
#[macro_use]
pub mod layout;

/// Module for registering global handlers / window handling
pub mod app;
/// Module for various common widgets (button, text, canvas, etc.)
pub mod widgets;
/// Drawing functions for images, rectangles, text and ellipses
pub mod draw;
/// UI state handling, such as adding / removing widgets,
pub mod ui;
/// Points, rects and sizes definitions
pub mod geometry;
/// Font, image and texture resources
pub mod resources;
/// Color constants. TODO: use CSS instead
pub mod color;
/// Keyboard and mouse handling (from `winit`)
pub mod input;
/// Re-exports of common crate-internal functions / structs
pub mod prelude;
/// Debugging rendering functions
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
