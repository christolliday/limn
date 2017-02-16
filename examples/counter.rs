extern crate limn;
extern crate glutin;

mod util;

use limn::widget::{EventHandler, EventArgs};
use limn::widget::builder::WidgetBuilder;
use limn::widget::layout::{LinearLayout, Orientation};
use limn::widgets::text::{self, TextDrawState, TextStyleField};
use limn::widget::style::Value;
use limn::widgets::button::PushButtonBuilder;
use limn::event::EventAddress;
use limn::color::*;
use limn::event::id::*;

struct CounterEvent(());
struct CountEvent(u32);

fn main() {
    let (window, ui, event_queue) = util::init_default("Limn counter demo");
    util::load_default_font();

    let mut root_widget = WidgetBuilder::new();

    let mut linear_layout = LinearLayout::new(Orientation::Horizontal, &mut root_widget);
    let mut left_spacer = WidgetBuilder::new();
    left_spacer.layout.width(50.0);
    linear_layout.add_widget(&mut left_spacer);
    root_widget.add_child(left_spacer);

    struct CountHandler {}
    impl EventHandler<CountEvent> for CountHandler {
        fn handle(&mut self, event: &CountEvent, args: EventArgs) {
            if let Some(drawable) = args.drawable.as_mut() {
                let &CountEvent(count) = event;
                drawable.update(|state: &mut TextDrawState| state.text = format!("{}", count));
            }
        }
    }

    let text_style = vec!{
        TextStyleField::Text(Value::Single("0".to_owned())),
        TextStyleField::BackgroundColor(Value::Single(WHITE)),
    };
    let text_drawable = text::text_drawable(text_style);
    let text_dims = text::measure(&text_drawable);
    let mut text_widget = WidgetBuilder::new()
        .set_drawable(text_drawable)
        .add_handler(CountHandler {});
    text_widget.layout.width(80.0);
    text_widget.layout.height(text_dims.height);
    text_widget.layout.center_vertical(&root_widget);
    linear_layout.add_widget(&mut text_widget);

    let mut button_container = WidgetBuilder::new();
    linear_layout.add_widget(&mut button_container);
    let root_id = root_widget.id;
    let mut button_widget = PushButtonBuilder::new()
        .set_text("Count").widget
        .on_click(move |_, args| {
            args.event_queue.push(EventAddress::Widget(root_id), NONE, CounterEvent(()));
        });
    button_widget.layout.center(&button_container);
    button_widget.layout.bound_by(&button_container, Some(50.0));
    button_container.add_child(button_widget);
    root_widget.add_child(text_widget);
    root_widget.add_child(button_container);

    struct CounterHandler {
        count: u32,
    }
    impl CounterHandler {
        fn new() -> Self {
            CounterHandler { count: 0 }
        }
    }
    impl EventHandler<CounterEvent> for CounterHandler {
        fn handle(&mut self, _: &CounterEvent, args: EventArgs) {
            self.count += 1;
            let address = EventAddress::SubTree(args.widget_id);
            args.event_queue.push(address, NONE, CountEvent(self.count));
        }
    }
    let root_widget = root_widget.add_handler(CounterHandler::new());

    util::set_root_and_loop(window, ui, root_widget, event_queue, vec!{});
}
