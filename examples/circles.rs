extern crate limn;
extern crate glutin;
extern crate graphics;
extern crate petgraph;
extern crate cassowary;

mod util;

use cassowary::strength::*;

use limn::widget::WidgetBuilder;
use limn::widget::WidgetBuilderCore;
use limn::widget::property::{Property, PropChange};
use limn::widgets::button::{PushButtonBuilder, WidgetClickable};
use limn::drawable::ellipse::EllipseDrawable;
use limn::event::Target;
use limn::ui::{self, Ui};
use limn::util::{Dimensions, Point};
use limn::resources::WidgetId;
use limn::color::*;

enum CircleEvent {
    Add(Point),
    Undo,
    Redo,
}

fn main() {
    let (window, mut ui) = util::init_default("Limn circles demo");
    util::load_default_font();

    fn create_undo_redo_buttons(root_widget: &mut WidgetBuilder) -> (WidgetId, WidgetId) {
        let mut button_container = WidgetBuilder::new();
        button_container.layout().center_horizontal(&root_widget.layout());
        button_container.layout().align_bottom(&root_widget.layout()).padding(20.0);
        button_container.layout().shrink();

        let mut undo_widget = PushButtonBuilder::new();
        undo_widget
            .set_text("Undo")
            .set_inactive()
            .on_click(|_, args| { args.queue.push(Target::Ui, CircleEvent::Undo); });

        let mut redo_widget = PushButtonBuilder::new();
        redo_widget
            .set_text("Redo")
            .set_inactive()
            .on_click(|_, args| { args.queue.push(Target::Ui, CircleEvent::Redo); });
        redo_widget.layout().to_right_of(&undo_widget.layout()).padding(20.0);

        let (undo_id, redo_id) = (undo_widget.id(), redo_widget.id());
        button_container.add_child(undo_widget);
        button_container.add_child(redo_widget);

        root_widget.add_child(button_container);
        (undo_id, redo_id)
    }

    fn create_circle(ui: &mut Ui, center: &Point) -> WidgetId {
        let border = graphics::ellipse::Border {
            color: BLACK,
            radius: 2.0,
        };
        let mut widget = WidgetBuilder::new();
        widget.set_drawable(EllipseDrawable::new(RED, Some(border)));
        widget.layout().dimensions(Dimensions {
            width: 30.0,
            height: 30.0,
        });
        let top_left = Point {
            x: center.x - 15.0,
            y: center.y - 15.0,
        };

        widget.layout().top_left(top_left).strength(STRONG);
        let id = widget.id();
        let root_id = ui.graph.root_id;
        ui.add_widget(widget, Some(root_id));
        id
    }

    struct CircleEventHandler {
        undo_id: WidgetId,
        redo_id: WidgetId,
        circles: Vec<(Point, WidgetId)>,
        undo: Vec<Point>,
    }
    impl CircleEventHandler {
        fn new(undo_id: WidgetId, redo_id: WidgetId) -> Self {
            CircleEventHandler {
                circles: Vec::new(),
                undo: Vec::new(),
                undo_id: undo_id,
                redo_id: redo_id,
            }
        }
    }
    impl ui::EventHandler<CircleEvent> for CircleEventHandler {
        fn handle(&mut self, event: &CircleEvent, args: ui::EventArgs) {
            match *event {
                CircleEvent::Add(point) => {
                    self.circles.push((point, create_circle(args.ui, &point)));
                    self.undo.clear();

                    args.queue.push(Target::SubTree(self.undo_id), PropChange::Remove(Property::Inactive));
                    args.queue.push(Target::SubTree(self.redo_id), PropChange::Add(Property::Inactive));
                }
                CircleEvent::Undo => {
                    if self.circles.len() > 0 {
                        let (point, widget_id) = self.circles.pop().unwrap();
                        args.ui.remove_widget(widget_id);
                        self.undo.push(point);
                        args.queue.push(Target::SubTree(self.redo_id), PropChange::Remove(Property::Inactive));
                        if self.circles.len() == 0 {
                            args.queue.push(Target::SubTree(self.undo_id), PropChange::Add(Property::Inactive));
                        }
                    }
                }
                CircleEvent::Redo => {
                    if self.undo.len() > 0 {
                        let point = self.undo.pop().unwrap();
                        self.circles.push((point, create_circle(args.ui, &point)));
                        if self.undo.len() == 0 {
                            args.queue.push(Target::SubTree(self.redo_id), PropChange::Add(Property::Inactive));
                        }
                    }
                }
            }
        }
    }
    let mut root_widget = WidgetBuilder::new();
    root_widget.on_click(|event, args| {
        let event = CircleEvent::Add(event.position);
        args.queue.push(Target::Ui, event);
    });
    root_widget.layout().dimensions(Dimensions {
        width: 300.0,
        height: 300.0,
    });


    let (undo_id, redo_id) = create_undo_redo_buttons(&mut root_widget);

    ui.add_handler(CircleEventHandler::new(undo_id, redo_id));

    util::set_root_and_loop(window, ui, root_widget);
}
