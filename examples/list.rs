extern crate limn;
extern crate input;

mod util;

use input::EventId;

use limn::widget::builder::WidgetBuilder;
use limn::widget::layout::{LinearLayout, Orientation};
use limn::widget::{EventHandler, EventArgs};
use limn::widgets::text::{self, TextDrawable};
use limn::widgets::primitives::{self, RectDrawable};
use limn::widgets::scroll::{ScrollHandler, WidgetScrollHandler};
use limn::event;
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

    let mut list_widget = WidgetBuilder::new();
    list_widget.layout.match_width(&scroll_widget.layout);
    list_widget.event_handlers.push(Box::new(WidgetScrollHandler::new()));

    struct MouseOnHandler {}
    impl EventHandler for MouseOnHandler {
        fn event_id(&self) -> EventId {
            event::WIDGET_MOUSE_OVER
        }
        fn handle_event(&mut self, mut args: EventArgs) {
            args.state.update(|state: &mut RectDrawable| state.background = BLUE);
        }
    }
    struct MouseOffHandler {}
    impl EventHandler for MouseOffHandler {
        fn event_id(&self) -> EventId {
            event::WIDGET_MOUSE_OFF
        }
        fn handle_event(&mut self, mut args: EventArgs) {
            args.state.update(|state: &mut RectDrawable| state.background = WHITE);
        }
    }
    struct MouseOnTextHandler {}
    impl EventHandler for MouseOnTextHandler {
        fn event_id(&self) -> EventId {
            event::WIDGET_MOUSE_OVER
        }
        fn handle_event(&mut self, mut args: EventArgs) {
            args.state.update(|state: &mut TextDrawable| state.text_color = WHITE);
        }
    }
    struct MouseOffTextHandler {}
    impl EventHandler for MouseOffTextHandler {
        fn event_id(&self) -> EventId {
            event::WIDGET_MOUSE_OFF
        }
        fn handle_event(&mut self, mut args: EventArgs) {
            args.state.update(|state: &mut TextDrawable| state.text_color = BLACK);
        }
    }

    let list_item_widgets = {
        let mut linear_layout = LinearLayout::new(Orientation::Vertical, &list_widget.layout);
        let mut list_item_widgets = Vec::new();
        for i in 1..15 {
            let text_drawable = TextDrawable::new_default("hello".to_owned(), font_id);
            let text_dims = text_drawable.measure_dims_no_wrap();

            let rect_drawable = RectDrawable { background: WHITE };
            let mut list_item_widget = WidgetBuilder::new()
                .set_drawable(primitives::draw_rect, Box::new(rect_drawable))
                .set_debug_name("item")
                .add_handler(Box::new(MouseOnHandler{}))
                .add_handler(Box::new(MouseOffHandler{}));
            list_item_widget.layout.match_width(&list_widget.layout);
            list_item_widget.layout.height(text_dims.height);
            linear_layout.add_widget(&mut list_item_widget.layout);

            let mut list_text_widget = WidgetBuilder::new()
                .set_drawable(text::draw_text, Box::new(text_drawable))
                .set_debug_name("text")
                .add_handler(Box::new(MouseOnTextHandler{}))
                .add_handler(Box::new(MouseOffTextHandler{}));
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
