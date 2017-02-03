extern crate limn;
extern crate glutin;
extern crate graphics;
extern crate petgraph;

mod util;

use petgraph::graph::NodeIndex;

use limn::widget::builder::WidgetBuilder;
use limn::widgets::button::PushButtonBuilder;
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

    fn create_undo_redo_buttons(ui: &mut Ui) -> NodeIndex {
        let button_container = {
            let root_widget = ui.get_root();
            let mut button_container = WidgetBuilder::new();
            button_container.layout.center_horizontal(&root_widget.layout);
            button_container.layout.align_bottom(&root_widget.layout, Some(20.0));
            //button_container.layout.height(40.0);

            let mut undo_widget = PushButtonBuilder::new()
                .set_text("Undo").widget;
            let mut redo_widget = PushButtonBuilder::new()
                .set_text("Redo").widget;
            redo_widget.layout.to_right_of(&undo_widget.layout, Some(20.0));
            
            button_container.add_child(Box::new(undo_widget));
            button_container.add_child(Box::new(redo_widget));
            button_container
        };
        let root_index = ui.root_index.unwrap();
        println!("making undo/redo");
        button_container.create(ui, Some(root_index))
    }
    fn create_circle(ui: &mut Ui, center: &Point) -> NodeIndex {

        let border = graphics::ellipse::Border { color: BLACK, radius: 2.0 };
        let mut widget = WidgetBuilder::new()
                        .set_drawable(primitives::ellipse_drawable(RED, Some(border)));
        widget.layout.dimensions(Dimensions {width: 30.0, height: 30.0});
        let top_left = Point { x: center.x - 15.0, y: center.y - 15.0 };

        widget.layout.top_left(top_left);
        let root_index = ui.root_index.unwrap();
        widget.create(ui, Some(root_index))
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
    struct AddCircleHandler {
        circles: Vec<NodeIndex>,
    }
    impl AddCircleHandler {
        fn new() -> Self {
            AddCircleHandler { circles: Vec::new() }
        }
    }
    impl UiEventHandler for AddCircleHandler {
        fn event_id(&self) -> EventId {
            ADD_CIRCLE
        }
        fn handle_event(&mut self, args: UiEventArgs) {
            let mouse = args.data.downcast_ref::<Point>().unwrap();
            if self.circles.len() == 0 {
                create_undo_redo_buttons(args.ui);
            }
            self.circles.push(create_circle(args.ui, mouse));
        }
    }
    let mut root_widget = WidgetBuilder::new()
        .add_handler(Box::new(CircleHandler{}));
    root_widget.layout.dimensions(Dimensions {width: 300.0, height: 300.0});

    let ui_event_handlers: Vec<Box<UiEventHandler>> = vec!{Box::new(AddCircleHandler::new())};
    util::set_root_and_loop(window, ui, root_widget, event_queue, ui_event_handlers);
}
