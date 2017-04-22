use glutin;

use event::{Target, UiEventHandler, WidgetEventArgs};
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

pub enum DragInputEvent {
    WidgetPressed(WidgetId),
    MouseMoved(Point),
    MouseReleased,
}

fn drag_handle_mouse_press(event: &WidgetMouseButton, args: WidgetEventArgs) {
    if let &WidgetMouseButton(glutin::ElementState::Pressed, _) = event {
        let event = DragInputEvent::WidgetPressed(args.widget.id);
        event!(Target::Ui, event);
    }
}
fn drag_handle_mouse_move(event: &MouseMoved, _: &mut Ui) {
    event!(Target::Ui, DragInputEvent::MouseMoved(event.0));
}
fn drag_handle_mouse_release(event: &MouseButton, _: &mut Ui) {
    if let &MouseButton(glutin::ElementState::Released, _) = event {
        event!(Target::Ui, DragInputEvent::MouseReleased);
    }
}

impl WidgetBuilder {
    pub fn make_draggable(&mut self) -> &mut Self {
        self.as_mut().add_handler_fn(drag_handle_mouse_press);
        self
    }
}

impl App {
    pub fn add_drag_handlers(&mut self) {
        self.add_handler(DragInputHandler::new());
        self.add_handler_fn(drag_handle_mouse_move);
        self.add_handler_fn(drag_handle_mouse_release);
    }
}
