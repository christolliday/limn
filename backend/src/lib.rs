//! Functionality for simplifying the work involved when using conrod along-side piston.

extern crate shader_version;
extern crate input as piston_input;
extern crate graphics as piston_graphics;

extern crate glutin_window;
use glutin_window::GlutinWindow;

pub mod window;
pub mod gfx;
pub mod events;

pub use self::window::{Window, WindowEvents};
pub use self::shader_version::OpenGL;
pub use self::gfx::GfxContext;