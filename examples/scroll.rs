#[macro_use]
extern crate limn;

mod util;

use limn::prelude::*;

use limn::widgets::scroll::ScrollBuilder;
use limn::draw::rect::{RectState, RectComponentStyle};

fn main() {
    let window_builder = glutin::WindowBuilder::new()
        .with_title("Limn scroll demo")
        .with_min_dimensions(300, 300);
    let app = util::init(window_builder);
    let mut root = WidgetBuilder::new("root");

    let mut scroll_widget = ScrollBuilder::new();
    scroll_widget.add_scrollbar();
    scroll_widget.layout().add(match_layout(&root).padding(50.0));

    let mut rect_container = WidgetBuilder::new("rect_container");
    rect_container.grid(3);
    rect_container.layout().add(size(Size::new(400.0, 400.0)));

    {
        let mut add_rect = |color| {
            let mut rect = WidgetBuilder::new(format!("rect_{:?}", color));
            rect.set_draw_style(RectComponentStyle {
                background_color: Some(Value::from(color)),
                ..RectComponentStyle::default()
            });
            rect_container.add_child(rect);
        };
        add_rect(GREEN);
        add_rect(BLUE);
        add_rect(RED);
        add_rect(YELLOW);
        add_rect(CYAN);
        add_rect(FUSCHIA);
        add_rect(WHITE);
        add_rect(GRAY_50);
        add_rect(BLACK);
    }

    scroll_widget.add_content(rect_container);
    root.add_child(scroll_widget);

    app.main_loop(root);
}
