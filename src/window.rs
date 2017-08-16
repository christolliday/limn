use gleam::gl;
use glutin;
use glutin::GlContext;
use webrender::api::{DeviceUintSize, LayoutSize};

pub struct Window {
    window: glutin::GlWindow
}
impl Window {
    pub fn new(title: &str, size: (u32, u32), min_size: Option<(u32, u32)>, events_loop: &glutin::EventsLoop) -> Self {
        let mut window = glutin::WindowBuilder::new()
            .with_title(title)
            .with_dimensions(size.0, size.1);

        if let Some(min_size) = min_size {
            window = window.with_min_dimensions(min_size.0, min_size.1)
        }
        let context = glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_gl(glutin::GlRequest::GlThenGles {
                opengl_version: (3, 2),
                opengles_version: (3, 0)
            });

        let window = glutin::GlWindow::new(window, context, &events_loop).unwrap();
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
    pub fn size_u32(&self) -> DeviceUintSize {
        let (width, height) = self.window.get_inner_size_pixels().unwrap();
        DeviceUintSize::new(width, height)
    }
    pub fn size_f32(&self) -> LayoutSize {
        let (width, height) = self.window.get_inner_size_pixels().unwrap();
        LayoutSize::new(width as f32, height as f32)
    }
}
