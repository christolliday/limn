#![deny(missing_docs)]

//! A Glutin window back-end for the Piston game engine.

use super::gl;
use super::glutin;

// External crates.
use super::input::{
    keyboard,
    MouseButton,
    Button,
    Input,
};
use super::pistoncore_window::{
    BuildFromWindowSettings,
    OpenGLWindow,
    Window,
    AdvancedWindow,
    ProcAddress,
    WindowSettings,
    Size,
    Position,
};
use glutin::{ Api, GlRequest };
use std::time::Duration;
use std::thread;

pub use shader_version::OpenGL;

/// Contains stuff for game window.
pub struct GlutinWindow {
    /// The window.
    pub window: glutin::Window,
    // The back-end does not remember the title.
    title: String,
    exit_on_esc: bool,
    should_close: bool,
    // Used to detect enter/leave cursor events.
    has_cursor: bool,
    // Used to fake capturing of cursor,
    // to get relative mouse events.
    is_capturing_cursor: bool,
    // Stores the last known cursor position.
    last_cursor_pos: Option<[i32; 2]>,
    // Stores relative coordinates to emit on next poll.
    mouse_relative: Option<(f64, f64)>,
    // Used to emit cursor event after enter/leave.
    cursor_pos: Option<[f64; 2]>,
    // Track the dimensions of the last `Resize` event that was emitted. Used for checking whether
    // or not we need to generate our own `Resize` events on systems that winit cannot generate
    // resize events for.
    last_resize_emitted_pixels: (u32, u32),
}

fn builder_from_settings(settings: &WindowSettings) -> glutin::WindowBuilder {
    let opengl = settings.get_maybe_opengl().unwrap_or(OpenGL::V3_2);
    let (major, minor) = opengl.get_major_minor();
    let size = settings.get_size();
    let mut builder = glutin::WindowBuilder::new()
        .with_min_dimensions(size.width, size.height)
        .with_dimensions(size.width, size.height)
        .with_decorations(settings.get_decorated())
        .with_multitouch()
        .with_gl(GlRequest::Specific(Api::OpenGl, (major as u8, minor as u8)))
        .with_title(settings.get_title())
        .with_srgb(Some(settings.get_srgb()));
    let samples = settings.get_samples();
    if settings.get_fullscreen() {
        builder = builder.with_fullscreen(glutin::get_primary_monitor());
    }
    if settings.get_vsync() {
        builder = builder.with_vsync();
    }
    if samples != 0 {
        builder = builder.with_multisampling(samples as u16);
    }
    builder
}

impl GlutinWindow {

    /// Creates a new game window for Glutin.
    pub fn new(settings: &WindowSettings) -> Result<Self, String> {
        use std::error::Error;
        use glutin::ContextError;

        let title = settings.get_title();
        let exit_on_esc = settings.get_exit_on_esc();
        let window = builder_from_settings(&settings).build();
        let window = match window {
                Ok(window) => window,
                Err(_) => {
                    try!(builder_from_settings(&settings.clone().samples(0)).build()
                        .map_err(|e| String::from(e.description()))
                    )
                }
            };
        unsafe { try!(window.make_current().map_err(|e|
                // This can be simplified in next version of Glutin.
                match e {
                    ContextError::IoError(ref err) => {
                        String::from(err.description())
                    }
                    ContextError::ContextLost => {
                        String::from("Context lost")
                    }
                }
            )); }

        // Load the OpenGL function pointers.
        gl::load_with(|s| window.get_proc_address(s) as *const _);

        let initial_dimensions = window.get_inner_size_pixels().unwrap_or((0, 0));

        Ok(GlutinWindow {
            window: window,
            title: title,
            exit_on_esc: exit_on_esc,
            should_close: false,
            has_cursor: true,
            cursor_pos: None,
            is_capturing_cursor: false,
            last_cursor_pos: None,
            mouse_relative: None,
            last_resize_emitted_pixels: initial_dimensions,
        })
    }

    fn wait_event(&mut self) -> Input {
        // First check for and handle any pending events.
        if let Some(event) = self.poll_event() {
            return event;
        }
        loop {
            if let Some(event) = self.check_for_new_resize_event() {
                return event;
            }
            let event = self.window.wait_events().next();
            if let Some(event) = self.handle_event(event) {
                return event;
            }
        }
    }

    fn wait_event_timeout(&mut self, timeout: Duration) -> Option<Input> {
        // First check for and handle any pending events.
        if let Some(event) = self.poll_event() {
            return Some(event);
        }
        // schedule wake up from `wait_event`
        let window_proxy = self.window.create_window_proxy();
        thread::spawn(move || {
            thread::sleep(timeout);
            window_proxy.wakeup_event_loop();
        });
        let event = self.window.wait_events().next();
        self.handle_event(event)
    }

    // Currently (12 Nov 2016) winit is unable to generate `Resize` events for Mac OS and possibly
    // other OSs, and shows no sign of producing a fix for this any time soon.
    //
    // Here, we get around this by keeping track of the last resize event that was emitted. Each
    // time a new event is requested, we can first compare the current `glutin::Window` size to the
    // last size emitted by a resize event. If the sizes do not match, we must generate our own
    // `Resize` event. Using this approach, we ensure that Mac OS users receive `Resize` events
    // while avoiding the creation of event doubles on OSs that already receive `Resize` events.
    fn resize_event_if_changed(&mut self, w: u32, h: u32) -> Option<Input> {
        let (last_w, last_h) = self.last_resize_emitted_pixels;
        if w != last_w || h != last_h {
            self.last_resize_emitted_pixels = (w, h);
            let dpi_factor = self.window.hidpi_factor();
            let w = (w as f32 / dpi_factor) as u32;
            let h = (h as f32 / dpi_factor) as u32;
            Some(Input::Resize(w, h))
        } else {
            None
        }
    }

    // Check to see whether or not we need to generate a new `Resize` event.
    fn check_for_new_resize_event(&mut self) -> Option<Input> {
        if let Some((w, h)) = self.window.get_inner_size_pixels() {
            if let Some(e) = self.resize_event_if_changed(w, h) {
                return Some(e);
            }
        }
        None
    }

    fn poll_event(&mut self) -> Option<Input> {
        use glutin::Event as E;
        use input::{ Input, Motion };

        // Check for a pending mouse cursor move event.
        if let Some(pos) = self.cursor_pos {
            self.cursor_pos = None;
            return Some(Input::Move(Motion::MouseCursor(pos[0], pos[1])));
        }

        // Check for a pending relative mouse move event.
        if let Some((x, y)) = self.mouse_relative {
            self.mouse_relative = None;
            return Some(Input::Move(Motion::MouseRelative(x, y)));
        }

        // Check to see whether or not we need to generate a `Resize` event.
        if let Some(event) = self.check_for_new_resize_event() {
            return Some(event);
        }

        let mut ev = self.window.poll_events().next();

        if self.is_capturing_cursor &&
           self.last_cursor_pos.is_none() {
            if let Some(E::MouseMoved(x, y)) = ev {
                // Ignore this event since mouse positions
                // should not be emitted when capturing cursor.
                self.last_cursor_pos = Some([x, y]);
                ev = self.window.poll_events().next();
            }
        }
        self.handle_event(ev)
    }

    /// Convert an incoming Glutin event to Piston input.
    /// Update cursor state if necessary.
    fn handle_event(&mut self, ev: Option<glutin::Event>) -> Option<Input> {
        use glutin::Event as E;
        use glutin::MouseScrollDelta;
        use input::{ Key, Input, Motion };

        match ev {
            None => {
                if self.is_capturing_cursor {
                    self.fake_capture();
                }
                None
            }
            Some(E::Resized(w, h)) =>
                self.resize_event_if_changed(w, h),
            Some(E::ReceivedCharacter(ch)) => {
                let string = match ch {
                    // Ignore control characters and return ascii for Text event (like sdl2).
                    '\u{7f}' | // Delete
                    '\u{1b}' | // Escape
                    '\u{8}'  | // Backspace
                    '\r' | '\n' | '\t' => "".to_string(),
                    _ => ch.to_string()
                };
                Some(Input::Text(string))
            },
            Some(E::Focused(focused)) =>
                Some(Input::Focus(focused)),
            Some(E::KeyboardInput(glutin::ElementState::Pressed, _, Some(key))) => {
                let piston_key = map_key(key);
                if let (true, Key::Escape) = (self.exit_on_esc, piston_key) {
                    self.should_close = true;
                }
                Some(Input::Press(Button::Keyboard(piston_key)))
            },
            Some(E::KeyboardInput(glutin::ElementState::Released, _, Some(key))) =>
                Some(Input::Release(Button::Keyboard(map_key(key)))),
            Some(E::Touch(glutin::Touch { phase, location, id })) => {
                use glutin::TouchPhase;
                use input::{Touch, TouchArgs};

                Some(Input::Move(Motion::Touch(TouchArgs::new(
                    0, id as i64, [location.0, location.1], 1.0, match phase {
                        TouchPhase::Started => Touch::Start,
                        TouchPhase::Moved => Touch::Move,
                        TouchPhase::Ended => Touch::End,
                        TouchPhase::Cancelled => Touch::Cancel
                    }
                ))))
            }
            Some(E::MouseMoved(x, y)) => {
                if let Some(pos) = self.last_cursor_pos {
                    let dx = x - pos[0];
                    let dy = y - pos[1];
                    if self.is_capturing_cursor {
                        self.last_cursor_pos = Some([x, y]);
                        self.fake_capture();
                        // Skip normal mouse movement and emit relative motion only.
                        return Some(Input::Move(Motion::MouseRelative(dx as f64, dy as f64)));
                    }
                    // Send relative mouse movement next time.
                    self.mouse_relative = Some((dx as f64, dy as f64));
                }

                self.last_cursor_pos = Some([x, y]);
                let f = self.window.hidpi_factor();
                let x = x as f64 / f as f64;
                let y = y as f64 / f as f64;
                let size = self.size();
                let cursor_inside = x >= 0.0 && x < size.width as f64 &&
                                    y >= 0.0 && y < size.height as f64;
                if cursor_inside != self.has_cursor {
                    self.cursor_pos = Some([x, y]);
                    self.has_cursor = cursor_inside;
                    return Some(Input::Cursor(cursor_inside));
                }
                Some(Input::Move(Motion::MouseCursor(x, y)))
            }
            Some(E::MouseWheel(MouseScrollDelta::PixelDelta(x, y), _)) =>
                Some(Input::Move(Motion::MouseScroll(x as f64, y as f64))),
            Some(E::MouseWheel(MouseScrollDelta::LineDelta(x, y), _)) =>
                Some(Input::Move(Motion::MouseScroll(x as f64, y as f64))),
            Some(E::MouseInput(glutin::ElementState::Pressed, button)) =>
                Some(Input::Press(Button::Mouse(map_mouse(button)))),
            Some(E::MouseInput(glutin::ElementState::Released, button)) =>
                Some(Input::Release(Button::Mouse(map_mouse(button)))),
            Some(E::Closed) => {
                self.should_close = true;
                Some(Input::Close)
            }
            _ => None,
        }
    }

    fn fake_capture(&mut self) {
        if let Some(pos) = self.last_cursor_pos {
            // Fake capturing of cursor.
            let size = self.size();
            let cx = (size.width / 2) as i32;
            let cy = (size.height / 2) as i32;
            let dx = cx - pos[0];
            let dy = cy - pos[1];
            if dx != 0 || dy != 0 {
                if let Ok(_) = self.window.set_cursor_position(cx as i32, cy as i32) {
                    self.last_cursor_pos = Some([cx, cy]);
                }
            }
        }
    }
}

impl Window for GlutinWindow {
    type Event = Input;

    fn size(&self) -> Size {
        self.window.get_inner_size().unwrap_or((0, 0)).into()
    }
    fn draw_size(&self) -> Size {
        self.window.get_inner_size_pixels().unwrap_or((0, 0)).into()
    }
    fn should_close(&self) -> bool { self.should_close }
    fn set_should_close(&mut self, value: bool) { self.should_close = value; }
    fn swap_buffers(&mut self) { let _ = self.window.swap_buffers(); }
    fn wait_event(&mut self) -> Input { self.wait_event() }
    fn wait_event_timeout(&mut self, timeout: Duration) -> Option<Input> { self.wait_event_timeout(timeout) }
    fn poll_event(&mut self) -> Option<Input> { self.poll_event() }
}

impl BuildFromWindowSettings for GlutinWindow {
    fn build_from_window_settings(settings: &WindowSettings)
    -> Result<Self, String> {
        GlutinWindow::new(settings)
    }
}

impl AdvancedWindow for GlutinWindow {
    fn get_title(&self) -> String { self.title.clone() }
    fn set_title(&mut self, value: String) {
        self.title = value;
        self.window.set_title(&self.title);
    }
    fn get_exit_on_esc(&self) -> bool { self.exit_on_esc }
    fn set_exit_on_esc(&mut self, value: bool) { self.exit_on_esc = value; }
    fn set_capture_cursor(&mut self, value: bool) {
        use glutin::CursorState;

        // Normally we would call `.set_cursor_state(CursorState::Grab)`
        // but since relative mouse events does not work,
        // the capturing of cursor is faked by hiding the cursor
        // and setting the position to the center of window.
        self.is_capturing_cursor = value;
        if value {
            let _ = self.window.set_cursor_state(CursorState::Hide);
        } else {
            let _ = self.window.set_cursor_state(CursorState::Normal);
        }
        if value {
            self.fake_capture();
        }
    }
    fn show(&mut self) { self.window.show(); }
    fn hide(&mut self) { self.window.hide(); }
    fn get_position(&self) -> Option<Position> {
        self.window.get_position().map(|(x, y)|
            Position { x: x, y: y })
    }
    fn set_position<P: Into<Position>>(&mut self, pos: P) {
        let pos: Position = pos.into();
        self.window.set_position(pos.x, pos.y);
    }
}

impl OpenGLWindow for GlutinWindow {
    fn get_proc_address(&mut self, proc_name: &str) -> ProcAddress {
        self.window.get_proc_address(proc_name) as *const _
    }

    fn is_current(&self) -> bool {
        self.window.is_current()
    }

    fn make_current(&mut self) {
        unsafe {
            self.window.make_current().unwrap()
        }
    }
}

/// Maps Glutin's key to Piston's key.
pub fn map_key(keycode: glutin::VirtualKeyCode) -> keyboard::Key {
    use input::keyboard::Key;
    use glutin::VirtualKeyCode as K;

    match keycode {
        K::Key0 => Key::D0,
        K::Key1 => Key::D1,
        K::Key2 => Key::D2,
        K::Key3 => Key::D3,
        K::Key4 => Key::D4,
        K::Key5 => Key::D5,
        K::Key6 => Key::D6,
        K::Key7 => Key::D7,
        K::Key8 => Key::D8,
        K::Key9 => Key::D9,
        K::A => Key::A,
        K::B => Key::B,
        K::C => Key::C,
        K::D => Key::D,
        K::E => Key::E,
        K::F => Key::F,
        K::G => Key::G,
        K::H => Key::H,
        K::I => Key::I,
        K::J => Key::J,
        K::K => Key::K,
        K::L => Key::L,
        K::M => Key::M,
        K::N => Key::N,
        K::O => Key::O,
        K::P => Key::P,
        K::Q => Key::Q,
        K::R => Key::R,
        K::S => Key::S,
        K::T => Key::T,
        K::U => Key::U,
        K::V => Key::V,
        K::W => Key::W,
        K::X => Key::X,
        K::Y => Key::Y,
        K::Z => Key::Z,
        K::Apostrophe => Key::Unknown,
        K::Backslash => Key::Backslash,
        K::Back => Key::Backspace,
        // K::CapsLock => Key::CapsLock,
        K::Delete => Key::Delete,
        K::Comma => Key::Comma,
        K::Down => Key::Down,
        K::End => Key::End,
        K::Return => Key::Return,
        K::Equals => Key::Equals,
        K::Escape => Key::Escape,
        K::F1 => Key::F1,
        K::F2 => Key::F2,
        K::F3 => Key::F3,
        K::F4 => Key::F4,
        K::F5 => Key::F5,
        K::F6 => Key::F6,
        K::F7 => Key::F7,
        K::F8 => Key::F8,
        K::F9 => Key::F9,
        K::F10 => Key::F10,
        K::F11 => Key::F11,
        K::F12 => Key::F12,
        K::F13 => Key::F13,
        K::F14 => Key::F14,
        K::F15 => Key::F15,
        // K::F16 => Key::F16,
        // K::F17 => Key::F17,
        // K::F18 => Key::F18,
        // K::F19 => Key::F19,
        // K::F20 => Key::F20,
        // K::F21 => Key::F21,
        // K::F22 => Key::F22,
        // K::F23 => Key::F23,
        // K::F24 => Key::F24,
        // Possibly next code.
        // K::F25 => Key::Unknown,
        K::Numpad0 => Key::NumPad0,
        K::Numpad1 => Key::NumPad1,
        K::Numpad2 => Key::NumPad2,
        K::Numpad3 => Key::NumPad3,
        K::Numpad4 => Key::NumPad4,
        K::Numpad5 => Key::NumPad5,
        K::Numpad6 => Key::NumPad6,
        K::Numpad7 => Key::NumPad7,
        K::Numpad8 => Key::NumPad8,
        K::Numpad9 => Key::NumPad9,
        K::NumpadComma => Key::NumPadDecimal,
        K::Divide => Key::NumPadDivide,
        K::Multiply => Key::NumPadMultiply,
        K::Subtract => Key::NumPadMinus,
        K::Add => Key::NumPadPlus,
        K::NumpadEnter => Key::NumPadEnter,
        K::NumpadEquals => Key::NumPadEquals,
        K::LShift => Key::LShift,
        K::LControl => Key::LCtrl,
        K::LAlt => Key::LAlt,
        K::LMenu => Key::LGui,
        K::RShift => Key::RShift,
        K::RControl => Key::RCtrl,
        K::RAlt => Key::RAlt,
        K::RMenu => Key::RGui,
        // Map to backslash?
        // K::GraveAccent => Key::Unknown,
        K::Home => Key::Home,
        K::Insert => Key::Insert,
        K::Left => Key::Left,
        K::LBracket => Key::LeftBracket,
        // K::Menu => Key::Menu,
        K::Minus => Key::Minus,
        K::Numlock => Key::NumLockClear,
        K::PageDown => Key::PageDown,
        K::PageUp => Key::PageUp,
        K::Pause => Key::Pause,
        K::Period => Key::Period,
        // K::PrintScreen => Key::PrintScreen,
        K::Right => Key::Right,
        K::RBracket => Key::RightBracket,
        // K::ScrollLock => Key::ScrollLock,
        K::Semicolon => Key::Semicolon,
        K::Slash => Key::Slash,
        K::Space => Key::Space,
        K::Tab => Key::Tab,
        K::Up => Key::Up,
        // K::World1 => Key::Unknown,
        // K::World2 => Key::Unknown,
        _ => Key::Unknown,
    }
}

/// Maps Glutin's mouse button to Piston's mouse button.
pub fn map_mouse(mouse_button: glutin::MouseButton) -> MouseButton {
    use glutin::MouseButton as M;

    match mouse_button {
        M::Left => MouseButton::Left,
        M::Right => MouseButton::Right,
        M::Middle => MouseButton::Middle,
        M::Other(0) => MouseButton::X1,
        M::Other(1) => MouseButton::X2,
        M::Other(2) => MouseButton::Button6,
        M::Other(3) => MouseButton::Button7,
        M::Other(4) => MouseButton::Button8,
        _ => MouseButton::Unknown
    }
}
