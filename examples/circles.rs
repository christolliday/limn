extern crate limn;
extern crate glutin;
extern crate graphics;

mod util;

use limn::widget::builder::WidgetBuilder;
use limn::widgets::button::ToggleButtonBuilder;
use limn::widgets::primitives;
use limn::widget::{EventHandler, EventArgs};
use limn::event::{EventId, EventAddress, WIDGET_PRESS};
use limn::ui::{Ui, UiEventHandler, UiEventArgs};
use limn::util::{Dimensions, Point};
use limn::color::*;

const ADD_CIRCLE: EventId = EventId("ADD_CIRCLE");

fn main() {
    let (window, ui, event_queue) = util::init_default("Limn circles demo");
    let font_id = util::load_default_font();

    fn create_circle(ui: &mut Ui, center: &Point) {

        let border = graphics::ellipse::Border { color: BLACK, radius: 2.0 };
        let mut widget = WidgetBuilder::new()
                        .set_drawable(primitives::ellipse_drawable(RED, Some(border)));
        widget.layout.dimensions(Dimensions {width: 30.0, height: 30.0});
        let top_left = Point { x: center.x - 15.0, y: center.y - 15.0 };

        widget.layout.top_left(top_left);
        let root_index = ui.root_index.unwrap();
        widget.create(ui, Some(root_index));
    }
    struct CircleHandler {}
    impl EventHandler for CircleHandler {
        fn event_id(&self) -> EventId {
            WIDGET_PRESS
        }
        fn handle_event(&mut self, args: EventArgs) {
            let event = args.data.downcast_ref::<glutin::Event>().unwrap();
            match *event {
                glutin::Event::MouseInput(state, button) => {
                    match state {
                        glutin::ElementState::Released => {
                            args.event_queue.push(EventAddress::Ui, ADD_CIRCLE, Box::new(args.input_state.mouse));
                        } _ => ()
                    }
                } _ => ()
            }
        }
    }
    struct AddCircleHandler {}
    impl UiEventHandler for AddCircleHandler {
        fn event_id(&self) -> EventId {
            ADD_CIRCLE
        }
        fn handle_event(&mut self, args: UiEventArgs) {
            let mouse = args.data.downcast_ref::<Point>().unwrap();
            create_circle(args.ui, mouse);
        }
    }
    let mut root_widget = WidgetBuilder::new()
        .add_handler(Box::new(CircleHandler{}));
    root_widget.layout.dimensions(Dimensions {width: 300.0, height: 300.0});

    let ui_event_handlers: Vec<Box<UiEventHandler>> = vec!{Box::new(AddCircleHandler {})};
    util::set_root_and_loop(window, ui, root_widget, event_queue, ui_event_handlers);
}
