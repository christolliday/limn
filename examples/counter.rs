#[macro_use]
extern crate limn;

mod util;

use limn::prelude::*;

use limn::widgets::text::StaticTextStyle;
use limn::widgets::button::ButtonStyle;
use limn::draw::text::TextState;

struct CountEvent;

fn main() {
    let window_builder = glutin::WindowBuilder::new()
        .with_title("Limn counter demo")
        .with_min_dimensions(100, 100);
    let app = util::init(window_builder);

    let mut root = WidgetBuilder::new("root");
    root.layout().add(min_size(Size::new(200.0, 100.0)));
    let mut layout_settings = LinearLayoutSettings::new(Orientation::Horizontal);
    layout_settings.spacing = Spacing::Around;
    root.linear_layout(layout_settings);

    #[derive(Default)]
    struct CountHandler {
        count: u32,
    }
    impl EventHandler<CountEvent> for CountHandler {
        fn handle(&mut self, _: &CountEvent, mut args: EventArgs) {
            self.count += 1;
            args.widget.update(|state: &mut TextState| state.text = format!("{}", self.count));
        }
    }

    let mut text_widget = StaticTextStyle::default();
    text_widget.text("0");
    let mut text_widget = WidgetBuilder::from_component_style(text_widget);
    text_widget.add_handler(CountHandler::default());
    text_widget.layout().add(constraints![
        center_vertical(&root),
    ]);

    let mut button_widget = ButtonStyle::default();
    button_widget.text("Count");
    let mut button_widget = WidgetBuilder::from_component_style(button_widget);
    let text_widget_ref = text_widget.widget_ref();
    button_widget.add_handler(move |_: &ClickEvent, _: EventArgs| {
        text_widget_ref.event(CountEvent);
    });
    button_widget.layout().add(constraints![
        center_vertical(&root),
    ]);
    root
        .add_child(text_widget)
        .add_child(button_widget);

    app.main_loop(root);
}
