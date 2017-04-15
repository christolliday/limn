#[macro_use]
extern crate limn;
extern crate glutin;

mod util;

use limn::widget::{WidgetBuilder, WidgetBuilderCore};
use limn::widgets::button::{PushButtonBuilder, WidgetClickable};
use limn::drawable::text::{TextDrawable, TextStyleable};
use limn::event::{Target, WidgetEventHandler, WidgetEventArgs};
use limn::color::*;

struct CounterEvent;
struct CountEvent(u32);

fn main() {
    let (window, ui) = util::init_default("Limn counter demo");
    util::load_default_font();

    let mut root_widget = WidgetBuilder::new();
    root_widget.hbox();

    let mut left_spacer = WidgetBuilder::new();
    left_spacer.layout().width(50.0);
    root_widget.add_child(left_spacer);

    struct CountHandler;
    impl WidgetEventHandler<CountEvent> for CountHandler {
        fn handle(&mut self, event: &CountEvent, args: WidgetEventArgs) {
            let &CountEvent(count) = event;
            args.widget.update(|state: &mut TextDrawable| state.text = format!("{}", count));
        }
    }

    let text_style = style!(TextStyleable::BackgroundColor: WHITE);
    let text_drawable = TextDrawable::new("0");
    let text_dims = text_drawable.measure();
    let mut text_widget = WidgetBuilder::new();
    text_widget
        .set_drawable_with_style(text_drawable, text_style)
        .add_handler(CountHandler);
    text_widget.layout().width(80.0);
    text_widget.layout().height(text_dims.height);
    text_widget.layout().center_vertical(&root_widget.layout());

    let mut button_container = WidgetBuilder::new();
    let root_id = root_widget.id();
    let mut button_widget = PushButtonBuilder::new();
    button_widget.set_text("Count");
    button_widget.on_click(move |_, args| {
        args.queue.push(Target::Widget(root_id), CounterEvent);
    });
    button_widget.layout().center(&button_container.layout());
    button_widget.layout().bound_by(&button_container.layout()).padding(50.0);
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
    impl WidgetEventHandler<CounterEvent> for CounterHandler {
        fn handle(&mut self, _: &CounterEvent, args: WidgetEventArgs) {
            self.count += 1;
            let address = Target::SubTree(args.widget.id);
            args.queue.push(address, CountEvent(self.count));
        }
    }
    root_widget.add_handler(CounterHandler::new());

    util::set_root_and_loop(window, ui, root_widget);
}
