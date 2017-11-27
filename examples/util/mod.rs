extern crate env_logger;

use limn::prelude::*;
use limn::input::{EscKeyCloseHandler, DebugSettingsHandler};
use limn::resources;
use limn::resources::font::FontDescriptor;
use limn::draw::rect::RectStyle;
use limn::draw::text::TextStyle;
use limn::draw::ellipse::EllipseStyle;

pub fn default_style() {
    let mut res = resources::resources();

    res.font_loader.register_font_data(FontDescriptor::from_family("NotoSans"), include_bytes!("../../assets/fonts/NotoSans/NotoSans-Regular.ttf").to_vec());

    res.theme.register_type_style(EllipseStyle::default());
    res.theme.register_type_style(RectStyle::default());
    res.theme.register_type_style(style!(TextStyle {
        font: FontDescriptor::from_family("NotoSans"),
        font_size: 24.0,
        text_color: BLACK,
        background_color: TRANSPARENT,
        wrap: Wrap::Whitespace,
        align: Align::Start,
    }));
    res.theme.register_class_prop_style("static_text", INACTIVE.clone(), style!(TextStyle {
        text_color: GRAY_50,
    }));
    res.theme.register_class_style("list_item_rect", style!(RectStyle {
        background_color: GRAY_30,
    }));
    res.theme.register_class_prop_style("list_item_rect", SELECTED.clone(), style!(RectStyle {
        background_color: BLUE_HIGHLIGHT,
    }));
    res.theme.register_class_prop_style("list_item_rect", MOUSEOVER.clone(), style!(RectStyle {
        background_color: GRAY_60,
    }));
    res.theme.register_class_style("list_item_text", style!(TextStyle {
        text_color: WHITE,
    }));
    res.theme.register_class_style("button_rect", style!(RectStyle {
        background_color: GRAY_80,
        corner_radius: Some(5.0),
        border: Some((1.0, GRAY_40)),
    }));
    res.theme.register_class_prop_style("button_rect", INACTIVE.clone(), style!(RectStyle {
        background_color: GRAY_90,
        border: Some((1.0, GRAY_70)),
    }));
    res.theme.register_class_prop_style("button_rect", ACTIVATED_PRESSED.clone(), style!(RectStyle {
        background_color: GRAY_30,
    }));
    res.theme.register_class_prop_style("button_rect", ACTIVATED.clone(), style!(RectStyle {
        background_color: GRAY_40,
    }));
    res.theme.register_class_prop_style("button_rect", PRESSED.clone(), style!(RectStyle {
        background_color: GRAY_60,
    }));
    res.theme.register_class_prop_style("button_rect", MOUSEOVER.clone(), style!(RectStyle {
        background_color: GRAY_90,
    }));
}

// Initialize a limn App with common handlers and set up logger
pub fn init(window_builder: glutin::WindowBuilder) -> App {
    env_logger::init().unwrap();

    let events_loop = glutin::EventsLoop::new();
    let window = Window::new(window_builder, &events_loop);
    let mut app = App::new(window, events_loop);

    default_style();

    // Closes app on ESC key
    app.add_handler(EscKeyCloseHandler);
    // Toggles debug bounds drawing on F1 key
    app.add_handler(DebugSettingsHandler::new());
    app
}
