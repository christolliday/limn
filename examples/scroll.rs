#[macro_use]
extern crate limn;
extern crate cassowary;

mod util;

use cassowary::strength::*;
use cassowary::WeightedRelation::*;

use limn::layout::LAYOUT;
use limn::widget::{WidgetBuilder, WidgetBuilderCore};
use limn::widgets::scroll::ScrollBuilder;
use limn::drawable::rect::{RectDrawable, RectStyleable};
use limn::util::Dimensions;
use limn::color::*;
use limn::layout::constraint::*;

fn main() {
    let (window, ui) = util::init_default("Limn scroll demo");

    let mut root_widget = WidgetBuilder::new();

    let mut scroll_widget = ScrollBuilder::new();
    scroll_widget.set_debug_name("scroll");
    layout!(scroll_widget:
        dimensions(Dimensions {
            width: 200.0,
            height: 200.0,
        }),
        bound_by(&root_widget).padding(50.0));

    let mut rect_container_widget = WidgetBuilder::new();
    rect_container_widget
        .set_debug_name("rect_container");
    layout!(rect_container_widget: dimensions(Dimensions {
        width: 400.0,
        height: 400.0,
    }));

    let style = style!(RectStyleable::BackgroundColor: RED);
    let mut rect_tl_widget = WidgetBuilder::new();
    rect_tl_widget.set_drawable_with_style(RectDrawable::new(), style);
    layout!(rect_tl_widget:
        align_top(&rect_container_widget),
        align_left(&rect_container_widget));

    let style = style!(RectStyleable::BackgroundColor: GREEN);
    let mut rect_tr_widget = WidgetBuilder::new();
    rect_tr_widget.set_drawable_with_style(RectDrawable::new(), style);
    layout!(rect_tr_widget:
        to_right_of(&rect_tl_widget),
        align_top(&rect_container_widget),
        align_right(&rect_container_widget));

    let style = style!(RectStyleable::BackgroundColor: BLUE);
    let mut rect_bl_widget = WidgetBuilder::new();
    rect_bl_widget.set_drawable_with_style(RectDrawable::new(), style);
    layout!(rect_bl_widget:
        below(&rect_tl_widget),
        align_bottom(&rect_container_widget),
        align_left(&rect_container_widget));

    let style = style!(RectStyleable::BackgroundColor: FUSCHIA);
    let mut rect_br_widget = WidgetBuilder::new();
    rect_br_widget.set_drawable_with_style(RectDrawable::new(), style);
    layout!(rect_br_widget:
        below(&rect_tr_widget),
        to_right_of(&rect_bl_widget),
        align_bottom(&rect_container_widget),
        align_right(&rect_container_widget));

    layout!(rect_tl_widget:
        match_width(&rect_tr_widget),
        match_width(&rect_bl_widget),
        match_height(&rect_tr_widget),
        match_height(&rect_bl_widget),
    );
    layout!(rect_br_widget:
        match_height(&rect_tr_widget),
        match_height(&rect_bl_widget),
        match_width(&rect_tr_widget),
        match_width(&rect_bl_widget),
    );

    {
        let ref rect_tl_widget = rect_tl_widget.layout().vars;
        let ref rect_tr_widget = rect_tr_widget.layout().vars;
        let ref rect_bl_widget = rect_bl_widget.layout().vars;
        layout!(rect_container_widget:
            LAYOUT.width | EQ(REQUIRED) | rect_tl_widget.width + rect_tr_widget.width,
            LAYOUT.height | EQ(REQUIRED) | rect_tl_widget.height + rect_bl_widget.height,
        );
    }
    rect_container_widget
        .add_child(rect_tl_widget)
        .add_child(rect_tr_widget)
        .add_child(rect_bl_widget)
        .add_child(rect_br_widget);
    scroll_widget.add_child(rect_container_widget);
    root_widget.add_child(scroll_widget);

    util::set_root_and_loop(window, ui, root_widget);
}
