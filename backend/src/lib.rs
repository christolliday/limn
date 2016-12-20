extern crate shader_version;

extern crate window as pistoncore_window;
extern crate input;
extern crate graphics;
extern crate glutin_window;
extern crate texture;

extern crate gfx_core;
extern crate gfx_device_gl;
extern crate gfx_graphics;

pub mod window;
pub mod gfx;
pub mod events;
pub mod glyph;

pub use self::window::{Window, WindowEvents};
pub use self::shader_version::OpenGL;
pub use gfx::GfxContext;