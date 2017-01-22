#[macro_use]
extern crate limn;
extern crate input;

mod util;

use std::any::Any;

use input::EventId;

use limn::widget;
use limn::widget::text::TextDrawable;
use limn::widget::builder::WidgetBuilder;
use limn::widget::button::PushButtonBuilder;
use limn::widget::layout::{LinearLayout, Orientation};
use limn::event::{self, Event, Signal, EventAddress};
use limn::widget::{EventHandler, EventArgs};
use limn::resources::{Id, resources};
use limn::color::*;

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

    let text_drawable = TextDrawable {
        text: "0".to_owned(),
        font_id: font_id,
        font_size: 20.0,
        text_color: BLACK,
        background_color: WHITE,
    };
    let text_dims = text_drawable.measure_dims_no_wrap();
    let mut text_widget = WidgetBuilder::new();
    text_widget.set_drawable(widget::text::draw_text, Box::new(text_drawable));
    text_widget.layout.width(80.0);
    text_widget.layout.height(text_dims.height);
    text_widget.layout.center_vertical(&root_widget.layout);
    linear_layout.add_widget(&mut text_widget.layout);

    struct CountHandler {}
    impl EventHandler for CountHandler {
        fn event_id(&self) -> EventId {
            COUNT
        }
        fn handle_event(&mut self, event_args: EventArgs) {
            let EventArgs { event, state, .. } = event_args;
            let state = state.unwrap();
            let state = state.downcast_mut::<TextDrawable>().unwrap();
            let count: &u32 = event.event_data().unwrap().downcast_ref().unwrap();
            state.text = format!("{}", count);
        }
    }
    text_widget.event_handlers.push(Box::new(CountHandler {}));

    let mut button_container = WidgetBuilder::new();
    linear_layout.add_widget(&mut button_container.layout);
    struct PushButtonHandler {
        receiver_id: Id,
    }
    impl EventHandler for PushButtonHandler {
        fn event_id(&self) -> EventId {
            event::WIDGET_PRESS
        }
        fn handle_event(&mut self, event_args: EventArgs) {
            let EventArgs { event_queue, .. } = event_args;
            let event = Signal::new(COUNTER);
            event_queue.push(EventAddress::IdAddress("SELF".to_owned(), self.receiver_id.0),
                             Box::new(event));
        }
    }
    let mut button_widget = PushButtonBuilder::new();
    button_widget.set_text("Count", font_id);
    button_widget.widget.layout.center(&button_container.layout);
    button_widget.widget.layout.pad(50.0, &button_container.layout);
    button_widget.widget.event_handlers.push(Box::new(PushButtonHandler { receiver_id: root_widget.id }));
    button_container.add_child(Box::new(button_widget.builder()));
    root_widget.add_child(Box::new(text_widget));
    root_widget.add_child(Box::new(button_container));


    event!(CountEvent, u32);
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
        fn handle_event(&mut self, event_args: EventArgs) {
            let EventArgs { widget_id, event_queue, .. } = event_args;
            self.count += 1;
            let event = CountEvent::new(COUNT, self.count);
            event_queue.push(EventAddress::IdAddress("CHILDREN".to_owned(), widget_id.0),
                             Box::new(event));
        }
    }
    root_widget.event_handlers.push(Box::new(CounterHandler::new()));

    util::set_root_and_loop(window, ui, root_widget);
}
