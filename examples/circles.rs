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

const CIRCLE_EVENT: EventId = EventId("CIRCLE_EVENT");

fn main() {
    let (window, mut ui, event_queue) = util::init_default("Limn circles demo");
    let font_id = util::load_default_font();

    fn create_undo_redo_buttons(ui: &mut Ui, root_widget: &mut WidgetBuilder) {
        let button_container = {
            let mut button_container = WidgetBuilder::new();
            button_container.layout.center_horizontal(&root_widget.layout);
            button_container.layout.align_bottom(&root_widget.layout, Some(20.0));
            button_container.layout.minimize();

            struct UndoHandler {}
            impl EventHandler for UndoHandler {
                fn event_id(&self) -> EventId {
                    WIDGET_PRESS
                }
                fn handle_event(&mut self, mut args: EventArgs) {
                    let event = args.data.downcast_ref::<glutin::Event>().unwrap();
                    match *event {
                        glutin::Event::MouseInput(state, button) => {
                            match state {
                                glutin::ElementState::Released => {
                                    println!("UNDO");
                                    args.event_state.handled = true;
                                }, _ => ()
                            }
                        }, _ => ()
                    }
                }
            }
            struct RedoHandler {}
            impl EventHandler for RedoHandler {
                fn event_id(&self) -> EventId {
                    WIDGET_PRESS
                }
                fn handle_event(&mut self, mut args: EventArgs) {
                    let event = args.data.downcast_ref::<glutin::Event>().unwrap();
                    match *event {
                        glutin::Event::MouseInput(state, button) => {
                            match state {
                                glutin::ElementState::Released => {
                                    println!("REDO");
                                    args.event_state.handled = true;
                                }, _ => ()
                            }
                        }, _ => ()
                    }
                }
            }
            let mut undo_widget = PushButtonBuilder::new()
                .set_text("Undo").widget
                .add_handler(Box::new(UndoHandler{}));
            let mut redo_widget = PushButtonBuilder::new()
                .set_text("Redo").widget
                .add_handler(Box::new(RedoHandler{}));
            redo_widget.layout.to_right_of(&undo_widget.layout, Some(20.0));

            button_container.add_child(Box::new(undo_widget));
            button_container.add_child(Box::new(redo_widget));
            button_container
        };
        root_widget.add_child(Box::new(button_container));
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

    enum CircleEvent {
        Add(Point),
        Undo,
        Redo,
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
                            let event = CircleEvent::Add(args.input_state.mouse);
                            args.event_queue.push(EventAddress::Ui, CIRCLE_EVENT, Box::new(event));
                        } _ => ()
                    }
                } _ => ()
            }
        }
    }
    struct AddCircleHandler {
        circles: Vec<NodeIndex>,
        undo: Vec<Point>,
    }
    impl AddCircleHandler {
        fn new() -> Self {
            AddCircleHandler { circles: Vec::new(), undo: Vec::new() }
        }
    }
    impl UiEventHandler for AddCircleHandler {
        fn event_id(&self) -> EventId {
            CIRCLE_EVENT
        }
        fn handle_event(&mut self, args: UiEventArgs) {
            let event = args.data.downcast_ref::<CircleEvent>().unwrap();
            match *event {
                CircleEvent::Add(point) => {
                    self.circles.push(create_circle(args.ui, &point));
                }
                CircleEvent::Undo => {
                    // remove one
                }
                CircleEvent::Redo => {
                    // add from undo queue, if any
                    // todo
                }
            }
        }
    }
    let mut root_widget = WidgetBuilder::new()
        .add_handler(Box::new(CircleHandler{}));
    root_widget.layout.dimensions(Dimensions {width: 300.0, height: 300.0});


    create_undo_redo_buttons(&mut ui, &mut root_widget);

    let ui_event_handlers: Vec<Box<UiEventHandler>> = vec!{Box::new(AddCircleHandler::new())};
    util::set_root_and_loop(window, ui, root_widget, event_queue, ui_event_handlers);
}