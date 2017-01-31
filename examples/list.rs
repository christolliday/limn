extern crate limn;

mod util;

use limn::widget::builder::WidgetBuilder;
use limn::widget::layout::{LinearLayout, Orientation};
use limn::widget::{EventHandler, EventArgs, Property, WidgetNotifyEvent, ChangePropEvent, PropsChangeEventHandler};
use limn::widgets::text::{self, TextDrawable, TEXT_STYLE_DEFAULT};
use limn::widgets::primitives::{self, RectDrawable};
use limn::widgets::list::{ListHandler, ListItemHandler, LIST_ITEM_STYLE_DEFAULT};
use limn::widgets::scroll::{ScrollHandler, WidgetScrollHandler};
use limn::widgets::hover::HoverHandler;
use limn::resources::Id;
use limn::event::{self, EventId, EventAddress, Signal};
use limn::util::Dimensions;
use limn::color::*;


fn main() {
    let (window, ui) = util::init_default("Limn list demo");
    let font_id = util::load_default_font();

    let mut root_widget = WidgetBuilder::new();
    
    let mut scroll_widget = WidgetBuilder::new();
    scroll_widget.layout.pad(50.0, &root_widget.layout);
    scroll_widget.layout.dimensions(Dimensions { width: 300.0, height: 300.0 });
    scroll_widget.layout.scrollable = true;
    scroll_widget.event_handlers.push(Box::new(ScrollHandler {}));

    let mut list_widget = WidgetBuilder::new()
        .add_handler(Box::new(ListHandler::new()))
        .add_handler(Box::new(WidgetScrollHandler::new()));
    list_widget.layout.match_width(&scroll_widget.layout);


    let list_item_widgets = {
        let mut linear_layout = LinearLayout::new(Orientation::Vertical, &list_widget.layout);
        let mut list_item_widgets = Vec::new();
        for i in 1..15 {
            let text_drawable = TextDrawable::new_style(TEXT_STYLE_DEFAULT.clone().with_text("hello").with_text_color(WHITE));
            let text_dims = text_drawable.measure_dims_no_wrap();

            let rect_drawable = RectDrawable::new(&LIST_ITEM_STYLE_DEFAULT);
            let mut list_item_widget = WidgetBuilder::new()
                .set_drawable(primitives::draw_rect, Box::new(rect_drawable))
                .set_style(primitives::apply_rect_style, Box::new(LIST_ITEM_STYLE_DEFAULT.clone()))
                .set_debug_name("item")
                .add_handler(Box::new(HoverHandler{}))
                .add_handler(Box::new(PropsChangeEventHandler{}))
                .add_handler(Box::new(ListItemHandler::new(list_widget.id)));
            list_item_widget.layout.match_width(&list_widget.layout);
            list_item_widget.layout.height(text_dims.height);
            linear_layout.add_widget(&mut list_item_widget.layout);

            let mut list_text_widget = WidgetBuilder::new()
                .set_drawable(text::draw_text, Box::new(text_drawable))
                .set_debug_name("text");
            list_text_widget.layout.center(&list_item_widget.layout);
            list_item_widget.add_child(Box::new(list_text_widget));

            list_item_widgets.push(list_item_widget);
        }
        list_item_widgets
    };

    for list_item_widget in list_item_widgets {
        list_widget.add_child(Box::new(list_item_widget));
    }
    scroll_widget.add_child(Box::new(list_widget));
    root_widget.add_child(Box::new(scroll_widget));

    util::set_root_and_loop(window, ui, root_widget);
}
