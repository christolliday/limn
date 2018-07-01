//! Contains the `App` type, used to initialize and run a Limn application.

use std::time::{Instant, Duration};
use std::rc::Rc;
use std::cell::RefCell;

use glutin::{self, dpi::LogicalSize};

use window::Window;
use ui::Ui;
use input::InputEvent;
use widget::Widget;
use event::{self, EventHandler};
use geometry::Size;

/// The `App` type is just a thin wrapper around a `Ui` containing
/// the methods used to initialize and run an `App`.
///
/// The main difference between `App` and `Ui` is that `App` is available
/// while initializing your application, and `Ui` is available to
/// every event handler. As such, `App` should contain methods that
/// can't, or shouldn't be called while the `App` is running.
///
/// There should be only one `App` per Window.
pub struct App {
    /// The UI currently visible in the window.
    ui: Ui,
    /// Minimum time until the next frame is drawn, caps the UI to 60 FPS.
    next_frame_time: Instant,
    /// Source of `glutin` input events.
    events_loop: Rc<RefCell<glutin::EventsLoop>>,
    /// Used to ignore resize events before ui has been measured
    window_initialized: bool,
}

impl App {
    /// Creates a new `App` from an existing `Window`.
    /// Automatically initializes the default event handlers for a typical
    /// desktop app:
    ///
    /// `ui_handlers`, `layout_handlers`, `input_handlers`,
    /// `mouse_handlers`, `keyboard_handlers` and `drag_handlers`
    pub fn new(window: Window, events_loop: glutin::EventsLoop) -> Self {
        event::queue_set_events_loop(&events_loop);
        let ui = Ui::new(window, &events_loop);
        let mut app = App {
            ui: ui,
            next_frame_time: Instant::now(),
            events_loop: Rc::new(RefCell::new(events_loop)),
            window_initialized: false,
        };
        app.initialize_handlers();
        app
    }

    /// Initialize the handlers that are used in a typical desktop app.
    /// The handlers that make up the event flow in an application are configurable
    fn initialize_handlers(&mut self) {
        self.add_ui_handlers();
        self.add_layout_handlers();
        self.add_input_handlers();

        self.add_mouse_handlers();
        self.add_keyboard_handlers();
        self.add_drag_handlers();
    }

    fn handle_window_event(&mut self, event: glutin::Event) {
        debug!("handle window event {:?}", event);
        if let glutin::Event::WindowEvent { event, .. } = event {
            if let glutin::WindowEvent::Resized(LogicalSize {width, height}) = event {
                // ignore resize events before ui has been measured
                if self.window_initialized {
                    self.ui.window_resized(Size::new(width as f32, height as f32));
                }
            } else {
                self.ui.event(InputEvent(event));
            }
        }
    }

    /// Updates the UI and redraws the window (the applications main loop)
    pub fn main_loop(mut self, root: Widget) {
        self.ui.root.add_child(root);
        let events_loop = Rc::clone(&self.events_loop);
        let mut events_loop = events_loop.borrow_mut();

        // Handle set up events to allow layout to 'settle' and initialize
        // the window size to the initial layout size
        self.handle_events();
        self.ui.resize_window_to_fit();
        self.ui.window.borrow_mut().show();
        self.window_initialized = true;
        loop {
            if !self.ui.needs_redraw() && !self.ui.render.frame_ready() {
                events_loop.run_forever(|event| {
                    self.handle_window_event(event);
                    glutin::ControlFlow::Break
                });
            }
            events_loop.poll_events(|event| {
                self.handle_window_event(event);
            });
            self.handle_events();
            if self.ui.should_close() {
                self.ui.render.deinit();
                return;
            }
            let now = Instant::now();
            if now > self.next_frame_time {
                let frame_length = Duration::new(0, 1_000_000_000 / 60);
                if self.next_frame_time + frame_length > now {
                    self.next_frame_time = now + frame_length;
                } else {
                    self.next_frame_time += frame_length;
                }
                self.ui.draw_if_needed();
                self.ui.get_root().event(FrameEvent);
                self.handle_events();
            }
            self.ui.update();
        }
    }

    /// Handle all the pending events in the event queue
    fn handle_events(&mut self) {
        while let Some((event_address, type_id, data)) = event::queue_next() {
            self.ui.handle_event(event_address, type_id, data.as_ref());
        }
    }

    /// Add a new global event handler
    pub fn add_handler<E: 'static, T: EventHandler<E> + 'static>(&mut self, handler: T) -> &mut Self {
        self.ui.get_root().add_handler(handler);
        self
    }

    pub fn get_root(&self) -> Widget {
        self.ui.get_root()
    }

    pub fn window(&self) -> ::std::cell::Ref<Window> {
        self.ui.window.borrow()
    }
}

/// Event emitted after every frame is rendered.
///
/// To implement animation, add a handler for this event that calls
/// [`args.ui.redraw()`](../ui/struct.Ui.html#method.redraw) to draw a new frame.
#[derive(Debug, Copy, Clone)]
pub struct FrameEvent;
