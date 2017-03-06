extern crate limn;

mod util;

use limn::widget::builder::WidgetBuilder;
use limn::drawable::rect::{RectDrawable, RectStyleField};
use limn::widget::style::Value;
use limn::color::*;
use limn::util::Dimensions;


fn main() {
    let (window, ui) = util::init_default("Limn scroll demo");

    let mut root_widget = WidgetBuilder::new().set_debug_name("root");

    let mut scroll_widget = WidgetBuilder::new()
        .set_debug_name("scroll")
        .contents_scroll();
    scroll_widget.layout.dimensions(Dimensions {
        width: 200.0,
        height: 200.0,
    });
    scroll_widget.layout.bound_by(&root_widget, Some(50.0));

    let mut rect_container_widget = WidgetBuilder::new()
        .set_debug_name("rect_container")
        .scrollable();
    rect_container_widget.layout.dimensions(Dimensions {
        width: 400.0,
        height: 400.0,
    });

    let style = vec![RectStyleField::BackgroundColor(Value::Single(RED))];
    let mut rect_tl_widget = WidgetBuilder::new().set_drawable_with_style(RectDrawable::new(), style);
    rect_tl_widget.layout.dimensions(Dimensions {
        width: 200.0,
        height: 200.0,
    });
    rect_tl_widget.layout.align_top(&rect_container_widget, None);
    rect_tl_widget.layout.align_left(&rect_container_widget, None);

    let style = vec![RectStyleField::BackgroundColor(Value::Single(GREEN))];
    let mut rect_tr_widget = WidgetBuilder::new().set_drawable_with_style(RectDrawable::new(), style);
    rect_tr_widget.layout.dimensions(Dimensions {
        width: 200.0,
        height: 200.0,
    });
    rect_tr_widget.layout.align_top(&rect_container_widget, None);
    rect_tr_widget.layout.align_right(&rect_container_widget, None);

    let style = vec![RectStyleField::BackgroundColor(Value::Single(BLUE))];
    let mut rect_bl_widget = WidgetBuilder::new().set_drawable_with_style(RectDrawable::new(), style);
    rect_bl_widget.layout.dimensions(Dimensions {
        width: 200.0,
        height: 200.0,
    });
    rect_bl_widget.layout.align_bottom(&rect_container_widget, None);
    rect_bl_widget.layout.align_left(&rect_container_widget, None);

    let style = vec![RectStyleField::BackgroundColor(Value::Single(FUSCHIA))];
    let mut rect_br_widget = WidgetBuilder::new().set_drawable_with_style(RectDrawable::new(), style);
    rect_br_widget.layout.dimensions(Dimensions {
        width: 200.0,
        height: 200.0,
    });
    rect_br_widget.layout.align_bottom(&rect_container_widget, None);
    rect_br_widget.layout.align_right(&rect_container_widget, None);

    rect_container_widget.add_child(rect_tl_widget);
    rect_container_widget.add_child(rect_tr_widget);
    rect_container_widget.add_child(rect_bl_widget);
    rect_container_widget.add_child(rect_br_widget);
    scroll_widget.add_child(rect_container_widget);
    root_widget.add_child(scroll_widget);

    util::set_root_and_loop(window, ui, root_widget);
}
