//! Limn is a cross platform, event driven, component based GUI library.

// ---- START CLIPPY CONFIG

#![cfg_attr(all(not(test), feature="clippy"), warn(result_unwrap_used))]
#![cfg_attr(feature="clippy", warn(unseparated_literal_suffix))]
#![cfg_attr(feature="clippy", warn(wrong_pub_self_convention))]

// Enable clippy if our Cargo.toml file asked us to do so.
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

#![warn(missing_copy_implementations,
        trivial_numeric_casts,
        trivial_casts,
        unused_extern_crates,
        unused_import_braces,
        unused_qualifications)]
#![cfg_attr(feature="clippy", warn(cast_possible_truncation))]
#![cfg_attr(feature="clippy", warn(cast_possible_wrap))]
#![cfg_attr(feature="clippy", warn(cast_precision_loss))]
#![cfg_attr(feature="clippy", warn(cast_sign_loss))]
#![cfg_attr(feature="clippy", warn(missing_docs_in_private_items))]
#![cfg_attr(feature="clippy", warn(mut_mut))]

// Disallow `println!`. Use `debug!` for debug output
// (which is provided by the `log` crate).
#![cfg_attr(feature="clippy", warn(print_stdout))]

// This allows us to use `unwrap` on `Option` values (because doing makes
// working with Regex matches much nicer) and when compiling in test mode
// (because using it in tests is idiomatic).
#![cfg_attr(all(not(test), feature="clippy"), warn(result_unwrap_used))]
#![cfg_attr(feature="clippy", warn(unseparated_literal_suffix))]
#![cfg_attr(feature="clippy", warn(wrong_pub_self_convention))]

// ---- END CLIPPY CONFIG

#![cfg_attr(feature="nightly", feature(core_intrinsics))]
#![cfg_attr(feature="nightly", feature(get_type_id))]

#[macro_use]
extern crate limn_layout;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate maplit;
#[macro_use]
extern crate log;

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

#[macro_use]
pub mod event;
/// Module for `WidgetBuilder` and callback handlers
#[macro_use]
pub mod widget;
/// Module for layout / resizing handlers and layout solving
#[macro_use]
pub mod layout;

pub mod app;
pub mod widgets;
pub mod draw;
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
pub mod style;

#[cfg(not(feature="nightly"))]
fn type_name<T>() -> &'static str {
    "Type unavailable, use nightly"
}

#[cfg(feature="nightly")]
fn type_name<T>() -> &'static str {
    unsafe { std::intrinsics::type_name::<T>() }
}
