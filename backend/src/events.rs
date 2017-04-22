use std::time::{Duration, Instant};
use std::thread;

use glutin;

#[derive(Debug)]
pub enum WindowEvent {
    Render,
    Input(glutin::Event),
}

pub struct WindowEvents {
    idle: bool,
    last_frame_time: Instant,
    next_frame_time: Instant,
    dt_frame: Duration,
}

static BILLION: u64 = 1_000_000_000;
trait UpdateDuration {
    fn from_freq(hz: u64) -> Duration;
    fn as_secs_f64(self) -> f64;
}
impl UpdateDuration for Duration {
    fn from_freq(hz: u64) -> Duration {
        let secs = (1.0 / hz as f64).floor() as u64;
        let nanos = ((BILLION / hz) % BILLION) as u32;
        Duration::new(secs, nanos)
    }
    fn as_secs_f64(self) -> f64 {
        self.as_secs() as f64 + self.subsec_nanos() as f64 / BILLION as f64
    }
}

/// The default maximum frames per second.
pub const DEFAULT_MAX_FPS: u64 = 60;

fn wait_event_timeout(window: &mut glutin::Window, timeout: Duration) -> Option<glutin::Event> {
    // First check for and handle any pending events.
    if let Some(event) = window.poll_events().next() {
        return Some(event);
    }
    // schedule wake up from `wait_event`
    let window_proxy = window.create_window_proxy();
    thread::spawn(move || {
        thread::sleep(timeout);
        window_proxy.wakeup_event_loop();
    });
    window.wait_events().next()
}

impl WindowEvents
{
    /// Creates a new event iterator
    pub fn new_with_fps(max_fps: u64) -> WindowEvents {
        let start = Instant::now();
        let frame_length = Duration::from_freq(max_fps);
        WindowEvents {
            idle: false,
            last_frame_time: start,
            next_frame_time: start,
            dt_frame: frame_length,
        }
    }
    /// Creates a new event iterator with default FPS settings.
    pub fn new() -> WindowEvents {
        WindowEvents::new_with_fps(DEFAULT_MAX_FPS)
    }

    /// Returns the next event.
    ///
    /// While in the `Waiting` state, returns `Input` events up until `dt_frame` has passed, or if idle, waits indefinitely.
    pub fn next(&mut self, window: &mut glutin::Window) -> WindowEvent
    {
        if self.idle {
            // Block and wait until an event is received.
            let event = wait_event_timeout(window, Duration::new(u64::max_value(), 0));
            self.idle = false;
            if let Some(event) = event {
                WindowEvent::Input(event)
            } else {
                WindowEvent::Render
            }
        } else {
            let current_time = Instant::now();
            if current_time < self.next_frame_time {
                // Wait for events until ready for next frame.
                let event = wait_event_timeout(window, self.next_frame_time - current_time);
                if let Some(event) = event {
                    return WindowEvent::Input(event);
                }
            }
            if let Some(event) = window.poll_events().next() {
                return WindowEvent::Input(event);
            }

            self.last_frame_time = Instant::now();
            self.next_frame_time = self.last_frame_time + self.dt_frame;
            self.idle = true;
            WindowEvent::Render
        }
    }
}
