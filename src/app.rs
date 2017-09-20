use std::time::{Instant, Duration};
use std::rc::Rc;
use std::cell::RefCell;

use glutin;

use window::Window;

use ui::Ui;
use input::InputEvent;
use widget::WidgetBuilder;
use event::{self, EventHandler, EventArgs};
use util::Size;

/// This is contains the core of a Limn application,
/// the Ui, event queue, and the handlers that operate
/// directly on the UI. These handlers are used to handle
/// global events and modify the UI graph.
/// A small set of handlers are configured by default that
/// are used in a typical desktop app. This set of handlers
/// could be configured differently for a mobile app, for example.
pub struct App {
    ui: Ui,
    next_frame_time: Instant,
    events_loop: Rc<RefCell<glutin::EventsLoop>>,
}

impl App {
    pub fn new(window: Window, events_loop: glutin::EventsLoop) -> Self {
        event::queue_set_events_loop(&events_loop);
        let ui = Ui::new(window, &events_loop);
        let mut app = App {
            ui: ui,
            next_frame_time: Instant::now(),
            events_loop: Rc::new(RefCell::new(events_loop)),
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
            if let glutin::WindowEvent::Resized(width, height) = event {
                self.ui.window_resized(Size::new(width as f32, height as f32));
            } else {
                self.ui.event(InputEvent(event));
            }
        }
    }
    /// Application main loop
    pub fn main_loop(mut self, root: WidgetBuilder) {
        self.ui.root.add_child(root);
        let events_loop = self.events_loop.clone();
        let mut events_loop = events_loop.borrow_mut();

        loop {
            events_loop.poll_events(|event| {
                self.handle_window_event(event);
            });
            if self.ui.should_close() {
                self.ui.render.deinit();
                return;
            }
            self.handle_events();
            let now = Instant::now();
            if now > self.next_frame_time {
                let frame_length = Duration::new(0, 1_000_000_000 / 60);
                if self.next_frame_time + frame_length > now {
                    self.next_frame_time = now + frame_length;
                } else {
                    self.next_frame_time += frame_length;
                }
                self.ui.draw_if_needed();
            }
            self.ui.update();

            if !self.ui.needs_redraw() && !self.ui.render.frame_ready() {
                let mut events = Vec::new();
                events_loop.run_forever(|window_event| {
                    events.push(window_event);
                    glutin::ControlFlow::Break
                });
                for event in events {
                    self.handle_window_event(event);
                }
            }
        }
    }

    /// Handle all the pending events in the event queue
    fn handle_events(&mut self) {
        while let Some((event_address, type_id, data)) = event::queue_next() {
            self.ui.handle_event(event_address, type_id, data.as_ref());
        }
    }

    /// Add a new stateful global event handler
    pub fn add_handler<E: 'static, T: EventHandler<E> + 'static>(&mut self, handler: T) -> &mut Self {
        self.ui.get_root().add_handler(handler);
        self
    }
    /// Add a new stateless global event handler
    pub fn add_handler_fn<E: 'static, T: Fn(&E, EventArgs) + 'static>(&mut self, handler: T) -> &mut Self {
        self.ui.get_root().add_handler_fn(handler);
        self
    }
}
