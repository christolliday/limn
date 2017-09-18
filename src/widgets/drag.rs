use glutin;

use event::{Target, UiEventHandler};
use widget::{WidgetBuilder, WidgetRef};
use input::mouse::{MouseMoved, MouseButton, WidgetMouseButton};
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
    widget: Option<WidgetRef>,
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
            DragInputEvent::WidgetPressed(ref widget) => {
                self.widget = Some(widget.clone());
                let event = WidgetDrag {
                    drag_type: DragEvent::DragStart,
                    position: self.position,
                };
                widget.event(event);
            }
            DragInputEvent::MouseReleased => {
                if let Some(widget) = self.widget.take() {
                    let event = WidgetDrag {
                        drag_type: DragEvent::DragEnd,
                        position: self.position,
                    };
                    widget.event(event);
                }
            }
            DragInputEvent::MouseMoved(point) => {
                self.position = point;
                if let Some(ref mut widget) = self.widget {
                    let event = WidgetDrag {
                        drag_type: DragEvent::Drag,
                        position: self.position,
                    };
                    widget.event(event);
                }
            }
        }
    }
}

enum DragInputEvent {
    WidgetPressed(WidgetRef),
    MouseMoved(Point),
    MouseReleased,
}

impl WidgetBuilder {
    pub fn make_draggable(&mut self) -> &mut Self {
        self.add_handler_fn(|event: &WidgetMouseButton, args| {
            if let &WidgetMouseButton(glutin::ElementState::Pressed, _) = event {
                let event = DragInputEvent::WidgetPressed(args.widget);
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
