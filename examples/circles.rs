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
use limn::widget::{EventHandler, EventArgs, Property};
use limn::event::{EventId, EventAddress, WIDGET_PRESS};
use limn::ui::{Ui, UiEventHandler, UiEventArgs};
use limn::util::{Dimensions, Point};
use limn::resources::WidgetId;
use limn::color::*;

const CIRCLE_EVENT: EventId = EventId("CIRCLE_EVENT");

fn main() {
    let (window, mut ui, mut event_queue) = util::init_default("Limn circles demo");
    let font_id = util::load_default_font();

    fn create_undo_redo_buttons(ui: &mut Ui, root_widget: &mut WidgetBuilder) -> (WidgetId, WidgetId) {
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

        let (undo_id, redo_id) = (undo_widget.id, redo_widget.id);
        button_container.add_child(Box::new(undo_widget));
        button_container.add_child(Box::new(redo_widget));

        root_widget.add_child(Box::new(button_container));
        (undo_id, redo_id)
    }

    fn create_circle(ui: &mut Ui, center: &Point) -> WidgetId {
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
        undo_id: WidgetId,
        redo_id: WidgetId,
        circles: Vec<(Point, WidgetId)>,
        undo: Vec<Point>,
    }
    impl CircleEventHandler {
        fn new(undo_id: WidgetId, redo_id: WidgetId) -> Self {
            CircleEventHandler { circles: Vec::new(), undo: Vec::new(), undo_id: undo_id, redo_id: redo_id }
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

                    args.event_queue.change_prop(self.undo_id, Property::Inactive, false);
                    args.event_queue.change_prop(self.redo_id, Property::Inactive, true);
                }
                CircleEvent::Undo => {
                    if self.circles.len() > 0 {
                        let (point, node_index) = self.circles.pop().unwrap();
                        args.ui.remove_widget(node_index);
                        self.undo.push(point);
                        if self.circles.len() == 0 {
                            args.event_queue.change_prop(self.undo_id, Property::Inactive, true);
                        }
                    }
                }
                CircleEvent::Redo => {
                    if self.undo.len() > 0 {
                        let point = self.undo.pop().unwrap();
                        self.circles.push((point, create_circle(args.ui, &point)));
                        if self.undo.len() == 0 {
                            args.event_queue.change_prop(self.redo_id, Property::Inactive, true);
                        }
                    }
                }
            }
        }
    }
    let mut root_widget = WidgetBuilder::new()
        .add_handler(Box::new(CircleClickHandler{}));
    root_widget.layout.dimensions(Dimensions {width: 300.0, height: 300.0});


    let (undo_id, redo_id) = create_undo_redo_buttons(&mut ui, &mut root_widget);
    // todo: better way to set initial props
    event_queue.change_prop(undo_id, Property::Inactive, true);
    event_queue.change_prop(redo_id, Property::Inactive, true);

    let ui_event_handlers: Vec<Box<UiEventHandler>> = vec!{Box::new(CircleEventHandler::new(undo_id, redo_id))};
    util::set_root_and_loop(window, ui, root_widget, event_queue, ui_event_handlers);
}
