extern crate limn;
extern crate glutin;

mod util;

use limn::widget::{WidgetBuilder, EventHandler, EventArgs};
use limn::widget::layout::{LinearLayout, Orientation};
use limn::widget::style::Value;
use limn::widgets::button::PushButtonBuilder;
use limn::drawable::text::{TextDrawable, TextStyleField};
use limn::event::Target;
use limn::color::*;

struct CounterEvent;
struct CountEvent(u32);

fn main() {
    let (window, ui) = util::init_default("Limn counter demo");
    util::load_default_font();

    let mut root_widget = WidgetBuilder::new();

    let mut linear_layout = LinearLayout::new(Orientation::Horizontal, &mut root_widget);
    let mut left_spacer = WidgetBuilder::new();
    left_spacer.layout.width(50.0);
    linear_layout.add_widget(&mut left_spacer);
    root_widget.add_child(left_spacer);

    struct CountHandler;
    impl EventHandler<CountEvent> for CountHandler {
        fn handle(&mut self, event: &CountEvent, args: EventArgs) {
            let &CountEvent(count) = event;
            args.widget.update(|state: &mut TextDrawable| state.text = format!("{}", count));
        }
    }

    let text_style = vec![TextStyleField::BackgroundColor(Value::Single(WHITE))];
    let text_drawable = TextDrawable::new("0");
    let text_dims = text_drawable.measure();
    let mut text_widget = WidgetBuilder::new();
    text_widget
        .set_drawable_with_style(text_drawable, text_style)
        .add_handler(CountHandler);
    text_widget.layout.width(80.0);
    text_widget.layout.height(text_dims.height);
    text_widget.layout.center_vertical(&root_widget.layout.vars);
    linear_layout.add_widget(&mut text_widget);

    let mut button_container = WidgetBuilder::new();
    linear_layout.add_widget(&mut button_container);
    let root_id = root_widget.id;
    let mut button_widget = PushButtonBuilder::new();
    button_widget.set_text("Count");
    let mut button_widget = button_widget.widget;
    button_widget.on_click(move |_, args| {
        args.queue.push(Target::Widget(root_id), CounterEvent);
    });
    button_widget.layout.center(&button_container.layout.vars);
    button_widget.layout.bound_by(&button_container.layout.vars, Some(50.0));
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
            let address = Target::SubTree(args.widget.id);
            args.queue.push(address, CountEvent(self.count));
        }
    }
    root_widget.add_handler(CounterHandler::new());

    util::set_root_and_loop(window, ui, root_widget);
}
