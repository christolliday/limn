extern crate limn;

mod util;

use limn::widget::builder::WidgetBuilder;
use limn::widgets::primitives::{self, RectDrawable};
use limn::widgets::scroll::{ScrollHandler, WidgetScrollHandler};
use limn::color::*;
use limn::util::Dimensions;


fn main() {
    let (window, ui) = util::init_default("Limn scroll demo");
    
    let mut root_widget = WidgetBuilder::new();

    let mut scroll_widget = WidgetBuilder::new();
    scroll_widget.layout.dimensions(Dimensions { width: 200.0, height: 200.0 });
    scroll_widget.layout.pad(100.0, &root_widget.layout);
    scroll_widget.layout.scrollable = true;
    scroll_widget.event_handlers.push(Box::new(ScrollHandler {}));

    let mut rect_container_widget = WidgetBuilder::new();
    rect_container_widget.event_handlers.push(Box::new(WidgetScrollHandler::new()));
    rect_container_widget.layout.dimensions(Dimensions { width: 400.0, height: 400.0});

    let mut rect_tl_widget = WidgetBuilder::new();
    rect_tl_widget.set_drawable(primitives::draw_rect, Box::new(RectDrawable { background: RED }));
    rect_tl_widget.layout.dimensions(Dimensions { width: 200.0, height: 200.0});
    rect_tl_widget.layout.align_top(&rect_container_widget.layout);
    rect_tl_widget.layout.align_left(&rect_container_widget.layout);

    let mut rect_tr_widget = WidgetBuilder::new();
    rect_tr_widget.set_drawable(primitives::draw_rect, Box::new(RectDrawable { background: GREEN }));
    rect_tr_widget.layout.dimensions(Dimensions { width: 200.0, height: 200.0});
    rect_tr_widget.layout.align_top(&rect_container_widget.layout);
    rect_tr_widget.layout.align_right(&rect_container_widget.layout);

    let mut rect_bl_widget = WidgetBuilder::new();
    rect_bl_widget.set_drawable(primitives::draw_rect, Box::new(RectDrawable { background: BLUE }));
    rect_bl_widget.layout.dimensions(Dimensions { width: 200.0, height: 200.0});
    rect_bl_widget.layout.align_bottom(&rect_container_widget.layout);
    rect_bl_widget.layout.align_left(&rect_container_widget.layout);

    let mut rect_br_widget = WidgetBuilder::new();
    rect_br_widget.set_drawable(primitives::draw_rect, Box::new(RectDrawable { background: FUSCHIA }));
    rect_br_widget.layout.dimensions(Dimensions { width: 200.0, height: 200.0});
    rect_br_widget.layout.align_bottom(&rect_container_widget.layout);
    rect_br_widget.layout.align_right(&rect_container_widget.layout);

    rect_container_widget.add_child(Box::new(rect_tl_widget));
    rect_container_widget.add_child(Box::new(rect_tr_widget));
    rect_container_widget.add_child(Box::new(rect_bl_widget));
    rect_container_widget.add_child(Box::new(rect_br_widget));
    scroll_widget.add_child(Box::new(rect_container_widget));
    root_widget.add_child(Box::new(scroll_widget));

    util::set_root_and_loop(window, ui, root_widget);
}
