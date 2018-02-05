#[allow(unused_imports)]
#[macro_use]
extern crate limn;

mod util;

use limn::prelude::*;

fn main() {
    let window_builder = glutin::WindowBuilder::new()
        .with_title("Limn scroll demo")
        .with_min_dimensions(300, 300);
    let app = util::init(window_builder);
    let mut root = Widget::new("root");

    let mut scroll_widget = ScrollContainer::default();
    scroll_widget.add_scrollbar();
    let mut rect_container = Widget::new("rect_container");
    rect_container.grid(3);
    rect_container.layout().add(size(Size::new(400.0, 400.0)));

    {
        let mut add_rect = |color| {
            let mut rect = Widget::new(format!("rect_{:?}", color));
            rect.set_draw_style(style!(RectStyle {
                background_color: color,
            }));
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
    let mut scroll_widget = Widget::from_modifier(scroll_widget);
    scroll_widget.layout().add(match_layout(&root).padding(50.0));
    root.add_child(scroll_widget);

    app.main_loop(root);
}
