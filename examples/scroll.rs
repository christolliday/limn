#[macro_use]
extern crate limn;
#[macro_use]
extern crate limn_layout;
extern crate cassowary;

mod util;

use limn::prelude::*;

use limn::widgets::scroll::ScrollBuilder;
use limn::drawable::rect::{RectDrawable, RectStyleable};

fn main() {
    let app = util::init_default_min_size("Limn scroll demo", Size::new(300.0, 300.0));

    let mut root_widget = WidgetRef::new();

    let mut scroll_widget = ScrollBuilder::new();
    scroll_widget.add_scrollbar();
    layout!(scroll_widget:
        match_layout(&root_widget).padding(50.0));

    let mut rect_container = WidgetRef::new_named("rect_container");
    rect_container.grid(3);
    layout!(rect_container: size(Size::new(400.0, 400.0)));

    {
        let mut add_rect = |color| {
            let mut rect = WidgetRef::new_named(&format!("rect_{:?}", color));
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
        add_rect(GRAY);
        add_rect(BLACK);
    }

    scroll_widget.add_content(rect_container);
    root_widget.add_child(scroll_widget);

    util::set_root_and_loop(app, root_widget);
}
