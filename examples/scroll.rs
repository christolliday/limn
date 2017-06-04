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
    let (window, ui) = util::init_default("Limn scroll demo");

    let mut root_widget = WidgetBuilder::new();

    let mut scroll_widget = ScrollBuilder::new();
    scroll_widget.add_scrollbar();
    layout!(scroll_widget:
        size(Size::new(200.0, 200.0)),
        match_layout(&root_widget).padding(50.0));

    let mut rect_container = WidgetBuilder::new_named("rect_container");
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

    layout!(rect_tl:
        align_top(&rect_container),
        align_left(&rect_container));

    layout!(rect_tr:
        to_right_of(&rect_tl),
        align_top(&rect_container),
        align_right(&rect_container),
        match_width(&rect_tl));

    layout!(rect_bl:
        below(&rect_tl),
        align_bottom(&rect_container),
        align_left(&rect_container),
        match_width(&rect_tl),
        match_height(&rect_tl));

    layout!(rect_br:
        below(&rect_tr),
        to_right_of(&rect_bl),
        align_bottom(&rect_container),
        align_right(&rect_container),
        match_width(&rect_bl),
        match_height(&rect_tr));

    rect_container
        .add_child(rect_tl)
        .add_child(rect_tr)
        .add_child(rect_bl)
        .add_child(rect_br);
    scroll_widget.add_content(rect_container);
    root_widget.add_child(scroll_widget);

    util::set_root_and_loop(window, ui, root_widget);
}
