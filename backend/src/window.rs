//! This code is adapted from the `piston_window` crate for convenience when
//! using the piston window API with a conrod app.
//!
//! Provides a simple API over the gfx graphics backend and the glutin window
//! context and events.
//!
//! Sets up:
//!
//! - [Gfx](https://github.com/gfx-rs/gfx) with an OpenGL back-end.
//! - [gfx_graphics](https://github.com/pistondevelopers/gfx_graphics)
//! for 2D rendering.
//! - [glutin_window](https://github.com/pistondevelopers/glutin_window)
//! as default window back-end, but this can be swapped (see below).
//!
//! ### Example
//!
//! ```no_run
//! extern crate conrod;
//! extern crate graphics;
//!
//! use conrod::backend::piston::{self, Window, WindowEvents};
//! use graphics::*;
//!
//! fn main() {
//!     let mut window: Window =
//!         piston::window::WindowSettings::new("Hello World!", [512; 2])
//!             .build().unwrap();
//!     let mut events = WindowEvents::new();
//!     while let Some(e) = window.next_event(&mut events) {
//!         window.draw_2d(&e, |c, g| {
//!             clear([0.5, 0.5, 0.5, 1.0], g);
//!             rectangle([1.0, 0.0, 0.0, 1.0], // red
//!                       [0.0, 0.0, 100.0, 100.0], // rectangle
//!                       c.transform, g);
//!         });
//!     }
//! }
//! ```
//!
//! ### Swap to another window back-end
//!
//! Change the generic parameter to the window back-end you want to use.
//!
//! ```ignore
//! extern crate conrod;
//! extern crate sdl2_window;
//!
//! use conrod::backend::piston::{self, Window};
//! use sdl2_window::Sdl2Window;
//!
//! # fn main() {
//!
//! let window: Window<Sdl2Window> =
//!     piston::window::WindowSettings::new("title", [512; 2])
//!         .build().unwrap();
//!
//! # }
//! ```
//!
//! ### sRGB
//!
//! The impl of `BuildFromWindowSettings` in this library turns on
//! `WindowSettings::srgb`, because it is required by gfx_graphics.
//!
//! Most images such as those found on the internet uses sRGB,
//! that has a non-linear gamma corrected space.
//! When rendering 3D, make sure textures and colors are in linear gamma space.
//! Alternative is to use `Srgb8` and `Srgba8` formats for textures.
//!
//! For more information about sRGB, see
//! https://github.com/PistonDevelopers/piston/issues/1014

extern crate window as pistoncore_window;

use input::{Event, GenericEvent, AfterRenderEvent};
use super::glutin_window::GlutinWindow;
use self::pistoncore_window::Window as BasicWindow;
use std::time::Duration;
use super::gfx::{GfxContext, G2d};
use super::shader_version::OpenGL;

pub use super::events::WindowEvents;
pub use self::pistoncore_window::{AdvancedWindow, Position, Size, OpenGLWindow, 
                                  WindowSettings, BuildFromWindowSettings};
use graphics::Viewport;

pub use super::graphics::Context;

/// Contains everything required for controlling window, graphics, event loop.
pub struct Window<W: BasicWindow = GlutinWindow> {
    /// The window.
    pub window: W,
    /// Stores state associated with Gfx.
    pub context: GfxContext,
}

impl<W> BuildFromWindowSettings for Window<W>
    where W: BasicWindow + OpenGLWindow + BuildFromWindowSettings,
          W::Event: GenericEvent
{
    fn build_from_window_settings(settings: &WindowSettings) -> Result<Window<W>, String> {
        // Turn on sRGB.
        let settings = settings.clone().srgb(true);

        let mut window: W = try!(settings.build());
        // Use OpenGL 3.2 by default, because this is what window backends usually do.
        let opengl = settings.get_maybe_opengl().unwrap_or(OpenGL::V3_2);
        let samples = settings.get_samples();
        let context = GfxContext::new(&mut window, opengl, samples);
        Ok(Window::new(window, context))
    }
}

impl<W> Window<W>
    where W: BasicWindow, W::Event: GenericEvent
{
    /// Creates a new piston window.
    pub fn new(window: W, context: GfxContext) -> Self
        where W: OpenGLWindow
    {
        Window {
            window: window,
            context: context,
        }
    }
    pub fn viewport(&self) -> Viewport {
        Viewport {
            rect: [0, 0, self.window.draw_size().width as i32, self.window.draw_size().height as i32],
            window_size: [self.window.size().width, self.window.size().height],
            draw_size: [self.window.draw_size().width, self.window.draw_size().height],
        }
    }

    /// Renders 2D graphics.
    pub fn draw_2d<F, U>(&mut self, f: F) -> U where
        W: OpenGLWindow,
        F: FnOnce(Context, &mut G2d) -> U
    {
        self.window.make_current();
        let viewport = self.viewport();
        self.context.draw_2d(f, viewport)
    }    

    /// Let window handle new event.
    pub fn handle_event(&mut self, event: &Event<W::Event>) {
        if let Some(_) = event.after_render_args() {
            self.swap_buffers();
            self.context.after_render();
        }
        self.context.check_resize(self.window.draw_size());
    }
}

impl<W> BasicWindow for Window<W>
    where W: BasicWindow
{
    type Event = <W as BasicWindow>::Event;

    fn should_close(&self) -> bool { self.window.should_close() }
    fn set_should_close(&mut self, value: bool) {
        self.window.set_should_close(value)
    }
    fn size(&self) -> Size { self.window.size() }
    fn draw_size(&self) -> Size { self.window.draw_size() }
    fn swap_buffers(&mut self) { self.window.swap_buffers() }
    fn wait_event(&mut self) -> Self::Event {
        BasicWindow::wait_event(&mut self.window)
    }
    fn wait_event_timeout(&mut self, timeout: Duration) -> Option<Self::Event> {
        BasicWindow::wait_event_timeout(&mut self.window, timeout)
    }
    fn poll_event(&mut self) -> Option<Self::Event> {
        BasicWindow::poll_event(&mut self.window)
    }
}

impl<W> AdvancedWindow for Window<W>
    where W: AdvancedWindow
{
    fn get_title(&self) -> String { self.window.get_title() }
    fn set_title(&mut self, title: String) {
        self.window.set_title(title)
    }
    fn get_exit_on_esc(&self) -> bool { self.window.get_exit_on_esc() }
    fn set_exit_on_esc(&mut self, value: bool) {
        self.window.set_exit_on_esc(value)
    }
    fn set_capture_cursor(&mut self, value: bool) {
        self.window.set_capture_cursor(value)
    }
    fn show(&mut self) { self.window.show() }
    fn hide(&mut self) { self.window.hide() }
    fn get_position(&self) -> Option<Position> {
        self.window.get_position()
    }
    fn set_position<P: Into<Position>>(&mut self, pos: P) {
        self.window.set_position(pos)
    }
}
