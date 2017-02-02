#[macro_use]
extern crate limn;

mod util;

use std::any::Any;

use limn::widget::{EventHandler, EventArgs};
use limn::widget::builder::WidgetBuilder;
use limn::widget::layout::{LinearLayout, Orientation};
use limn::widgets::text::{self, TextDrawState, TextStyleField, TextStyle};
use limn::widget::style::Value;
use limn::widgets::button::PushButtonBuilder;
use limn::event::{self, EventId, EventAddress};
use limn::resources::Id;
use limn::color::*;
use limn::theme::STYLE_TEXT;

const COUNTER: EventId = EventId("COUNTER");
const COUNT: EventId = EventId("COUNT");

fn main() {
    let (window, ui) = util::init_default("Limn counter demo");
    let font_id = util::load_default_font();

    let mut root_widget = WidgetBuilder::new();

    let mut linear_layout = LinearLayout::new(Orientation::Horizontal, &root_widget.layout);
    let mut left_spacer = WidgetBuilder::new();
    left_spacer.layout.width(50.0);
    linear_layout.add_widget(&mut left_spacer.layout);
    root_widget.add_child(Box::new(left_spacer));

    struct CountHandler {}
    impl EventHandler for CountHandler {
        fn event_id(&self) -> EventId {
            COUNT
        }
        fn handle_event(&mut self, args: EventArgs) {
            if let Some(drawable) = args.drawable.as_mut() {
                let count = args.data.downcast_ref::<u32>().unwrap();
                drawable.update(|state: &mut TextDrawState| state.text = format!("{}", count));
            }
        }
    }

    let text_fields = vec!{
        TextStyleField::text(Value::Single("0".to_owned())),
        TextStyleField::text_color(Value::Single(WHITE)),
    };
    let text_style = TextStyle::from(text_fields);
    let text_drawable = text::text_drawable(text_style);
    let text_dims = text::measure_dims_no_wrap(&text_drawable);
    let mut text_widget = WidgetBuilder::new()
        .set_drawable(text_drawable)
        .add_handler(Box::new(CountHandler {}));
    text_widget.layout.width(80.0);
    text_widget.layout.height(text_dims.height);
    text_widget.layout.center_vertical(&root_widget.layout);
    linear_layout.add_widget(&mut text_widget.layout);

    let mut button_container = WidgetBuilder::new();
    linear_layout.add_widget(&mut button_container.layout);
    struct PushButtonHandler {
        receiver_id: Id,
    }
    impl EventHandler for PushButtonHandler {
        fn event_id(&self) -> EventId {
            event::WIDGET_PRESS
        }
        fn handle_event(&mut self, args: EventArgs) {
            args.event_queue.push(EventAddress::Widget(self.receiver_id),
                                  COUNTER,
                                  Box::new(()));
        }
    }
    let mut button_widget = PushButtonBuilder::new()
        .set_text("Count", font_id)
        .widget
        .add_handler(Box::new(PushButtonHandler { receiver_id: root_widget.id }));
    button_widget.layout.center(&button_container.layout);
    button_widget.layout.pad(50.0, &button_container.layout);
    button_container.add_child(Box::new(button_widget));
    root_widget.add_child(Box::new(text_widget));
    root_widget.add_child(Box::new(button_container));

    struct CounterHandler {
        count: u32,
    }
    impl CounterHandler {
        fn new() -> Self {
            CounterHandler { count: 0 }
        }
    }
    impl EventHandler for CounterHandler {
        fn event_id(&self) -> EventId {
            COUNTER
        }
        fn handle_event(&mut self, args: EventArgs) {
            self.count += 1;
            args.event_queue.push(EventAddress::SubTree(args.widget_id),
                                  COUNT,
                                  Box::new(self.count));
        }
    }
    root_widget.event_handlers.push(Box::new(CounterHandler::new()));

    util::set_root_and_loop(window, ui, root_widget);
}
