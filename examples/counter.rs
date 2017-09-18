#[allow(unused_imports)]
#[macro_use]
extern crate limn;
#[macro_use]
extern crate limn_layout;

mod util;

use limn::prelude::*;

use limn::widgets::text::TextBuilder;
use limn::widgets::button::PushButtonBuilder;
use limn::drawable::text::TextDrawable;

struct CountEvent;

fn main() {
    let app = util::init_default("Limn counter demo");
    let mut root = WidgetBuilder::new("root");
    root.hbox();

    let mut left_spacer = WidgetBuilder::new("spacer");
    left_spacer.layout().add(width(50.0));
    root.add_child(left_spacer);

    #[derive(Default)]
    struct CountHandler {
        count: u32,
    }
    impl WidgetEventHandler<CountEvent> for CountHandler {
        fn handle(&mut self, _: &CountEvent, mut args: WidgetEventArgs) {
            self.count += 1;
            args.widget.update(|state: &mut TextDrawable| state.text = format!("{}", self.count));
        }
    }

    let mut text_widget = TextBuilder::new("0");
    text_widget.add_handler(CountHandler::default());
    text_widget.layout().add(center_vertical(&root));

    let mut button_container = WidgetBuilder::new("button_container");
    let mut button_widget = PushButtonBuilder::new();
    button_widget.set_text("Count");
    let text_widget_ref = text_widget.widget_ref();
    button_widget.on_click(move |_, _| {
        text_widget_ref.event(CountEvent);
    });
    button_widget.layout().add(constraints![
        center(&button_container),
        bound_by(&button_container).padding(50.0),
    ]);
    button_container.add_child(button_widget);
    root
        .add_child(text_widget)
        .add_child(button_container);

    app.main_loop(root);
}
