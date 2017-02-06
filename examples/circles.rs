extern crate limn;
extern crate glutin;
extern crate graphics;
extern crate petgraph;
extern crate cassowary;

mod util;

use petgraph::graph::NodeIndex;

use cassowary::strength::*;

use limn::widget::builder::WidgetBuilder;
use limn::widgets::button::PushButtonBuilder;
use limn::widgets::primitives;
use limn::widget::{EventHandler, EventArgs};
use limn::event::{EventId, EventAddress, WIDGET_PRESS};
use limn::ui::{Ui, UiEventHandler, UiEventArgs};
use limn::util::{Dimensions, Point};
use limn::resources::Id;
use limn::color::*;

const CIRCLE_EVENT: EventId = EventId("CIRCLE_EVENT");

fn main() {
    let (window, mut ui, event_queue) = util::init_default("Limn circles demo");
    let font_id = util::load_default_font();

    fn create_undo_redo_buttons(ui: &mut Ui, root_widget: &mut WidgetBuilder) {
        let button_container = {
            let mut button_container = WidgetBuilder::new();
            button_container.layout.center_horizontal(&root_widget);
            button_container.layout.align_bottom(&root_widget, Some(20.0));
            button_container.layout.minimize();

            let mut undo_widget = PushButtonBuilder::new()
                .set_text("Undo")
                .set_on_click(|args| {
                    args.event_queue.push(EventAddress::Ui, CIRCLE_EVENT, Box::new(CircleEvent::Undo));
                }).widget;
            let mut redo_widget = PushButtonBuilder::new()
                .set_text("Redo")
                .set_on_click(|args| {
                    args.event_queue.push(EventAddress::Ui, CIRCLE_EVENT, Box::new(CircleEvent::Redo));
                }).widget;
            redo_widget.layout.to_right_of(&undo_widget, Some(20.0));

            button_container.add_child(Box::new(undo_widget));
            button_container.add_child(Box::new(redo_widget));
            button_container
        };
        root_widget.add_child(Box::new(button_container));
    }

    fn create_circle(ui: &mut Ui, center: &Point) -> Id {
        let border = graphics::ellipse::Border { color: BLACK, radius: 2.0 };
        let mut widget = WidgetBuilder::new()
                        .set_drawable(primitives::ellipse_drawable(RED, Some(border)));
        widget.layout.dimensions(Dimensions {width: 30.0, height: 30.0});
        let top_left = Point { x: center.x - 15.0, y: center.y - 15.0 };

        widget.layout.top_left(top_left, Some(STRONG));
        let id = widget.id;
        let root_index = ui.root_index.unwrap();
        ui.add_widget(widget, Some(root_index));
        id
    }

    enum CircleEvent {
        Add(Point),
        Undo,
        Redo,
    }

    struct CircleClickHandler {}
    impl EventHandler for CircleClickHandler {
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
    struct CircleEventHandler {
        circles: Vec<(Point, Id)>,
        undo: Vec<Point>,
    }
    impl CircleEventHandler {
        fn new() -> Self {
            CircleEventHandler { circles: Vec::new(), undo: Vec::new() }
        }
    }
    impl UiEventHandler for CircleEventHandler {
        fn event_id(&self) -> EventId {
            CIRCLE_EVENT
        }
        fn handle_event(&mut self, args: UiEventArgs) {
            let event = args.data.downcast_ref::<CircleEvent>().unwrap();
            match *event {
                CircleEvent::Add(point) => {
                    self.circles.push((point, create_circle(args.ui, &point)));
                    self.undo.clear();
                }
                CircleEvent::Undo => {
                    if self.circles.len() > 0 {
                        let (point, node_index) = self.circles.pop().unwrap();
                        args.ui.remove_widget(node_index);
                        self.undo.push(point);
                    }
                }
                CircleEvent::Redo => {
                    if self.undo.len() > 0 {
                        let point = self.undo.pop().unwrap();
                        self.circles.push((point, create_circle(args.ui, &point)));
                    }
                }
            }
        }
    }
    let mut root_widget = WidgetBuilder::new()
        .add_handler(Box::new(CircleClickHandler{}));
    root_widget.layout.dimensions(Dimensions {width: 300.0, height: 300.0});


    create_undo_redo_buttons(&mut ui, &mut root_widget);

    let ui_event_handlers: Vec<Box<UiEventHandler>> = vec!{Box::new(CircleEventHandler::new())};
    util::set_root_and_loop(window, ui, root_widget, event_queue, ui_event_handlers);
}
