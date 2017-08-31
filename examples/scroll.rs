#[macro_use]
extern crate limn;

mod util;

use limn::prelude::*;

use limn::widgets::scroll::ScrollBuilder;
use limn::drawable::rect::{RectDrawable, RectStyleable};

fn main() {
    let app = util::init_default_min_size("Limn scroll demo", Size::new(300.0, 300.0));
    let mut root = app.ui.root.clone();

    let mut scroll_widget = ScrollBuilder::new();
    scroll_widget.add_scrollbar();
    scroll_widget.layout().add(match_layout(&root).padding(50.0));

    let mut rect_container = Widget::new_named("rect_container");
    rect_container.grid(3);
    rect_container.layout().add(size(Size::new(400.0, 400.0)));

    {
        let mut add_rect = |color| {
            let mut rect = Widget::new_named(&format!("rect_{:?}", color));
            rect.set_drawable_with_style(RectDrawable::new(),
                style!(RectStyleable::BackgroundColor: color));
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

    app.main_loop();
}
