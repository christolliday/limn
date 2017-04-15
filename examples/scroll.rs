#[macro_use]
extern crate limn;

mod util;

use limn::widget::{WidgetBuilder, WidgetBuilderCore};
use limn::drawable::rect::{RectDrawable, RectStyleField};
use limn::util::Dimensions;
use limn::color::*;

fn main() {
    let (window, ui) = util::init_default("Limn scroll demo");

    let mut root_widget = WidgetBuilder::new();

    let mut scroll_widget = WidgetBuilder::new();
    scroll_widget
        .set_debug_name("scroll")
        .contents_scroll();
    scroll_widget.layout().dimensions(Dimensions {
        width: 200.0,
        height: 200.0,
    });
    scroll_widget.layout().bound_by(&root_widget.layout()).padding(50.0);

    let mut rect_container_widget = WidgetBuilder::new();
    rect_container_widget
        .set_debug_name("rect_container")
        .make_scrollable();
    rect_container_widget.layout().dimensions(Dimensions {
        width: 400.0,
        height: 400.0,
    });

    let style = style!(RectStyleField::BackgroundColor: RED);
    let mut rect_tl_widget = WidgetBuilder::new();
    rect_tl_widget.set_drawable_with_style(RectDrawable::new(), style);
    rect_tl_widget.layout().align_top(&rect_container_widget.layout());
    rect_tl_widget.layout().align_left(&rect_container_widget.layout());

    let style = style!(RectStyleField::BackgroundColor: GREEN);
    let mut rect_tr_widget = WidgetBuilder::new();
    rect_tr_widget.set_drawable_with_style(RectDrawable::new(), style);
    rect_tr_widget.layout().to_right_of(&rect_tl_widget.layout());
    rect_tr_widget.layout().align_top(&rect_container_widget.layout());
    rect_tr_widget.layout().align_right(&rect_container_widget.layout());

    let style = style!(RectStyleField::BackgroundColor: BLUE);
    let mut rect_bl_widget = WidgetBuilder::new();
    rect_bl_widget.set_drawable_with_style(RectDrawable::new(), style);
    rect_bl_widget.layout().below(&rect_tl_widget.layout());
    rect_bl_widget.layout().align_bottom(&rect_container_widget.layout());
    rect_bl_widget.layout().align_left(&rect_container_widget.layout());

    let style = style!(RectStyleField::BackgroundColor: FUSCHIA);
    let mut rect_br_widget = WidgetBuilder::new();
    rect_br_widget.set_drawable_with_style(RectDrawable::new(), style);
    rect_br_widget.layout().below(&rect_tr_widget.layout());
    rect_br_widget.layout().to_right_of(&rect_bl_widget.layout());
    rect_br_widget.layout().align_bottom(&rect_container_widget.layout());
    rect_br_widget.layout().align_right(&rect_container_widget.layout());

    rect_tl_widget.layout().match_width(&rect_tr_widget.layout());
    rect_tl_widget.layout().match_height(&rect_bl_widget.layout());
    rect_br_widget.layout().match_height(&rect_tr_widget.layout());
    rect_br_widget.layout().match_width(&rect_bl_widget.layout());

    rect_container_widget.add_child(rect_tl_widget);
    rect_container_widget.add_child(rect_tr_widget);
    rect_container_widget.add_child(rect_bl_widget);
    rect_container_widget.add_child(rect_br_widget);
    scroll_widget.add_child(rect_container_widget);
    root_widget.add_child(scroll_widget);

    util::set_root_and_loop(window, ui, root_widget);
}
