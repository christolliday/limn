#[macro_use]
extern crate limn;
#[macro_use]
extern crate limn_layout;
extern crate cassowary;

mod util;

use limn::widget::{WidgetBuilder, WidgetBuilderCore};
use limn::widgets::scroll::ScrollBuilder;
use limn::drawable::rect::{RectDrawable, RectStyleable};
use limn::util::Size;
use limn::color::*;
use limn::layout::constraint::*;

fn main() {
    let (window, ui) = util::init_default_min_size("Limn scroll demo", Size::new(300.0, 300.0));

    let mut root_widget = WidgetBuilder::new();

    let mut scroll_widget = ScrollBuilder::new();
    scroll_widget.add_scrollbar();
    layout!(scroll_widget:
        match_layout(&root_widget).padding(50.0));

    let mut rect_container = WidgetBuilder::new_named("rect_container");
    rect_container.grid(2);
    layout!(rect_container: size(Size::new(400.0, 400.0)));

    let mut rect_tl = WidgetBuilder::new_named("rect_tl");
    rect_tl.set_drawable_with_style(RectDrawable::new(),
        style!(RectStyleable::BackgroundColor: RED));
    let mut rect_tr = WidgetBuilder::new_named("rect_tr");
    rect_tr.set_drawable_with_style(RectDrawable::new(),
        style!(RectStyleable::BackgroundColor: GREEN));
    let mut rect_bl = WidgetBuilder::new_named("rect_bl");
    rect_bl.set_drawable_with_style(RectDrawable::new(),
        style!(RectStyleable::BackgroundColor: BLUE));
    let mut rect_br = WidgetBuilder::new_named("rect_br");
    rect_br.set_drawable_with_style(RectDrawable::new(),
        style!(RectStyleable::BackgroundColor: FUSCHIA));

    rect_container
        .add_child(rect_tl)
        .add_child(rect_tr)
        .add_child(rect_bl)
        .add_child(rect_br);
    scroll_widget.add_content(rect_container);
    root_widget.add_child(scroll_widget);

    util::set_root_and_loop(window, ui, root_widget);
}
