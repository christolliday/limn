extern crate env_logger;

use limn::prelude::*;
use limn::input::{EscKeyCloseHandler, DebugSettingsHandler};
use limn::resources;
//use limn::widgets::button;
//use limn::widgets::text::StaticTextStyle;
use limn::draw::rect::RectComponentStyle;

pub fn default_style() {
    let mut res = resources::resources();
    res.theme.register_style_class("rect", RectComponentStyle {
        background_color: Some(Value::from(YELLOW)),
        ..RectComponentStyle::default()
        //style: Some(button::STYLE_BUTTON_TEXT.clone()),
    });
}

// Initialize a limn App with common handlers and set up logger
pub fn init(window_builder: glutin::WindowBuilder) -> App {
    default_style();

    env_logger::init().unwrap();
    let events_loop = glutin::EventsLoop::new();
    let window = Window::new(window_builder, &events_loop);
    let mut app = App::new(window, events_loop);

    // Closes app on ESC key
    app.add_handler(EscKeyCloseHandler);
    // Toggles debug bounds drawing on F1 key
    app.add_handler(DebugSettingsHandler::new());
    app
}
