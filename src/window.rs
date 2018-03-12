use gleam::gl;
use glutin;
use glutin::GlContext;
use webrender::api::DeviceUintSize;
use geometry::Size;

/// A simple wrapper around a `glutin::GlWindow`.
pub struct Window {
    pub window: glutin::GlWindow
}
impl Window {
    pub fn new(mut window_builder: glutin::WindowBuilder, events_loop: &glutin::EventsLoop) -> Self {
        let context = glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_gl(glutin::GlRequest::GlThenGles {
                opengl_version: (3, 2),
                opengles_version: (3, 0)
            });

        // create the window in a hidden state, so the window is first shown after being properly sized
        window_builder = window_builder.with_visibility(false);
        let window = glutin::GlWindow::new(window_builder, context, events_loop).unwrap();
        unsafe { window.make_current().ok() };
        Window {
            window: window
        }
    }
    pub fn gl(&self) -> ::std::rc::Rc<gl::Gl> {
        match gl::GlType::default() {
            gl::GlType::Gl => unsafe { gl::GlFns::load_with(|symbol| self.window.get_proc_address(symbol) as *const _) },
            gl::GlType::Gles => unsafe { gl::GlesFns::load_with(|symbol| self.window.get_proc_address(symbol) as *const _) },
        }
    }
    pub fn swap_buffers(&self) {
        self.window.swap_buffers().ok();
    }
    pub fn hidpi_factor(&self) -> f32 {
        self.window.hidpi_factor()
    }
    pub fn resize(&mut self, width: u32, height: u32) {
        self.window.set_inner_size(width, height);
    }
    /// Get the size of the client area of the window in actual pixels.
    /// This is the size of the framebuffer
    pub fn size_px(&self) -> DeviceUintSize {
        let (width, height) = self.window.get_inner_size().unwrap();
        DeviceUintSize::new(width, height)
    }
    /// Get the size of the client area of the window in density independent pixels.
    pub fn size_dp(&self) -> Size {
        let (width, height) = self.window.get_inner_size().unwrap();
        let hidpi = self.hidpi_factor();
        Size::new(width as f32 / hidpi, height as f32 / hidpi)
    }
    pub fn show(&self) {
        self.window.show()
    }
}
