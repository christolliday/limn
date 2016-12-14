extern crate backend;
extern crate graphics;

use backend::{Window, WindowEvents, OpenGL};
use graphics::*;

fn main() {
    const WIN_W: u32 = 400;
    const WIN_H: u32 = 720;

    // Construct the window.
    let mut window: Window =
        backend::window::WindowSettings::new("Grafiki Demo", [WIN_W, WIN_H])
            .opengl(OpenGL::V3_2).samples(4).exit_on_esc(true).build().unwrap();

    // Create the event loop.
    let mut events = WindowEvents::new();

    // Poll events from the window.
    while let Some(event) = events.next(&mut window) {
        window.handle_event(&event);

        window.draw_2d(&event, |c, g| {
            clear([0.8, 0.8, 0.8, 1.0], g);
            Rectangle::new([1.0, 0.0, 0.0, 1.0])
                .draw([0.0, 0.0, 100.0, 100.0], &c.draw_state, c.transform, g);

            Rectangle::new([0.5, 1.0, 0.0, 0.3])
                .draw([50.0, 50.0, 100.0, 100.0], &c.draw_state, c.transform, g);
        });
    }
}