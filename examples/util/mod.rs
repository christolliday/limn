extern crate env_logger;

use limn::prelude::*;
use limn::input::{EscKeyCloseHandler, DebugSettingsHandler};
use limn::resources;
use limn::draw::rect::RectComponentStyle;
use limn::draw::text::TextComponentStyle;
use limn::draw::ellipse::EllipseComponentStyle;

pub fn default_style() {
    let mut res = resources::resources();
    res.theme.register_type_style(EllipseComponentStyle::default());
    res.theme.register_type_style(RectComponentStyle::default());
    res.theme.register_type_style(TextComponentStyle {
        font: Some("NotoSans/NotoSans-Regular".to_owned()),
        font_size: Some(24.0),
        text_color: Some(BLACK),
        background_color: Some(TRANSPARENT),
        wrap: Some(Wrap::Whitespace),
        align: Some(Align::Start),
        ..TextComponentStyle::default()
    });
    res.theme.register_style_class("list_item_rect", RectComponentStyle {
        background_color: Some(GRAY_30),
        ..RectComponentStyle::default()
    });
    res.theme.register_style_class_prop("list_item_rect", SELECTED.clone(), RectComponentStyle {
        background_color: Some(BLUE_HIGHLIGHT),
        ..RectComponentStyle::default()
    });
    res.theme.register_style_class_prop("list_item_rect", MOUSEOVER.clone(), RectComponentStyle {
        background_color: Some(GRAY_60),
        ..RectComponentStyle::default()
    });
    res.theme.register_style_class("list_item_text", TextComponentStyle {
        text_color: Some(WHITE),
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
