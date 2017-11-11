extern crate env_logger;

use limn::prelude::*;
use limn::input::{EscKeyCloseHandler, DebugSettingsHandler};
use limn::resources;
use limn::draw::rect::RectComponentStyle;
use limn::draw::text::TextComponentStyle;

pub fn default_style() {
    let mut res = resources::resources();
    res.theme.register_style(TextComponentStyle {
        font: Some(Value::from("NotoSans/NotoSans-Regular".to_owned())),
        font_size: Some(Value::from(24.0)),
        text_color: Some(Value::from(BLACK)),
        background_color: Some(Value::from(TRANSPARENT)),
        wrap: Some(Value::from(Wrap::Whitespace)),
        align: Some(Value::from(Align::Start)),
        ..TextComponentStyle::default()
    });
    res.theme.register_style_class("list_item_rect", RectComponentStyle {
        background_color: Some(Value::from(selector!(GRAY_30,
            SELECTED: BLUE_HIGHLIGHT,
            MOUSEOVER: GRAY_60))),
        ..RectComponentStyle::default()
    });
    res.theme.register_style_class("list_item_text", TextComponentStyle {
        text_color: Some(Value::from(WHITE)),
        ..TextComponentStyle::default()
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
