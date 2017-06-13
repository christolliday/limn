use glutin;

use event::{Target, UiEventHandler};
use widget::{WidgetBuilder, WidgetBuilderCore};
use input::mouse::{MouseMoved, MouseButton, WidgetMouseButton};
use resources::WidgetId;
use util::Point;
use ui::Ui;
use app::App;

#[derive(Clone)]
pub struct WidgetDrag {
    pub drag_type: DragEvent,
    pub position: Point,
}

#[derive(Debug, Clone)]
pub enum DragEvent {
    DragStart,
    Drag,
    DragEnd,
}

struct DragInputHandler {
    widget: Option<WidgetId>,
    position: Point,
}
impl DragInputHandler {
    pub fn new() -> Self {
        DragInputHandler {
            widget: None,
            position: Point::new(0.0, 0.0),
        }
    }
}
impl UiEventHandler<DragInputEvent> for DragInputHandler {
    fn handle(&mut self, event: &DragInputEvent, _: &mut Ui) {
        match *event {
            DragInputEvent::WidgetPressed(id) => {
                self.widget = Some(id);
                let event = WidgetDrag {
                    drag_type: DragEvent::DragStart,
                    position: self.position,
                };
                event!(Target::Widget(id), event);
            }
            DragInputEvent::MouseReleased => {
                if let Some(id) = self.widget {
                    self.widget = None;
                    let event = WidgetDrag {
                        drag_type: DragEvent::DragEnd,
                        position: self.position,
                    };
                    event!(Target::Widget(id), event);
                }
            }
            DragInputEvent::MouseMoved(point) => {
                self.position = point;
                if let Some(id) = self.widget {
                    let event = WidgetDrag {
                        drag_type: DragEvent::Drag,
                        position: self.position,
                    };
                    event!(Target::Widget(id), event);
                }
            }
        }
    }
}

enum DragInputEvent {
    WidgetPressed(WidgetId),
    MouseMoved(Point),
    MouseReleased,
}

impl WidgetBuilder {
    pub fn make_draggable(&mut self) -> &mut Self {
        self.as_mut().add_handler_fn(|event: &WidgetMouseButton, args| {
            if let &WidgetMouseButton(glutin::ElementState::Pressed, _) = event {
                let event = DragInputEvent::WidgetPressed(args.widget.id);
                event!(Target::Ui, event);
            }
        });
        self
    }
}

impl App {
    pub fn add_drag_handlers(&mut self) {
        self.add_handler(DragInputHandler::new());
        self.add_handler_fn(|event: &MouseMoved, _| {
            event!(Target::Ui, DragInputEvent::MouseMoved(event.0));
        });
        self.add_handler_fn(|event: &MouseButton, _| {
            if let &MouseButton(glutin::ElementState::Released, _) = event {
                event!(Target::Ui, DragInputEvent::MouseReleased);
            }
        });
    }
}
