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

use glutin;

pub use super::graphics::Context;

/// Contains everything required for controlling window, graphics, event loop.
pub struct Window {
    /// The window.
    pub window: GlutinWindow,
    /// Stores state associated with Gfx.
    pub context: GfxContext,
}

impl Window {
    pub fn new<T, S>(title: T, size: S, min_size: Option<S>) -> Self 
    where T: Into<String>,
          S: Into<Size>,
    {
        let size: Size = size.into();
        
        let builder = glutin::WindowBuilder::new()
            .with_title(title)
            .with_dimensions(size.width, size.height);
        
        let builder = {
            if let Some(min_size) = min_size {
                let min_size: Size = min_size.into();
                builder.with_min_dimensions(min_size.width, min_size.height)
            } else {
                builder
            }
        };
        let mut glutin_window = GlutinWindow::new(builder).unwrap();

        let opengl = OpenGL::V3_2;
        let samples = 4;
        let context = GfxContext::new(&mut glutin_window, opengl, samples);

        Window {
            window: glutin_window,
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
        F: FnOnce(Context, &mut G2d) -> U
    {
        self.window.make_current();
        let viewport = self.viewport();
        let res = self.context.draw_2d(f, viewport);
        self.swap_buffers();
        self.context.after_render();
        res
    }    
    pub fn window_resized(&mut self) {
        self.context.check_resize(self.window.draw_size());
    }
}

impl BasicWindow for Window {
    type Event = <GlutinWindow as BasicWindow>::Event;

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