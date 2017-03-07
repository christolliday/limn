use glutin;

use widget::{EventArgs, EventHandler};
use ui;
use event::Target;
use util::Point;
use input::mouse::{MouseMoved, MouseButton, WidgetMouseButton};
use resources::WidgetId;

pub struct WidgetDrag {
    pub drag_type: DragEvent,
    pub position: Point,
}

#[derive(Debug)]
pub enum DragEvent {
    DragStart,
    Drag,
    DragEnd,
}

pub struct DragInputHandler {
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
impl ui::EventHandler<DragInputEvent> for DragInputHandler {
    fn handle(&mut self, event: &DragInputEvent, args: ui::EventArgs) {
        match *event {
            DragInputEvent::WidgetPressed(id) => {
                self.widget = Some(id);
                let event = WidgetDrag {
                    drag_type: DragEvent::DragStart,
                    position: self.position,
                };
                args.event_queue.push(Target::Widget(id), event);
            }
            DragInputEvent::MouseReleased => {
                if let Some(id) = self.widget {
                    self.widget = None;
                    let event = WidgetDrag {
                        drag_type: DragEvent::DragEnd,
                        position: self.position,
                    };
                    args.event_queue.push(Target::Widget(id), event);
                }
            }
            DragInputEvent::MouseMoved(point) => {
                self.position = point;
                if let Some(id) = self.widget {
                    let event = WidgetDrag {
                        drag_type: DragEvent::Drag,
                        position: self.position,
                    };
                    args.event_queue.push(Target::Widget(id), event);
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

pub struct DragWidgetPressHandler;
impl EventHandler<WidgetMouseButton> for DragWidgetPressHandler {
    fn handle(&mut self, event: &WidgetMouseButton, args: EventArgs) {
        let &WidgetMouseButton(state, _) = event;
        match state {
            glutin::ElementState::Pressed => {
                let event = DragInputEvent::WidgetPressed(args.widget.id);
                args.event_queue.push(Target::Ui, event);
            }
            _ => (),
        }
    }
}
pub struct DragMouseCursorHandler;
impl ui::EventHandler<MouseMoved> for DragMouseCursorHandler {
    fn handle(&mut self, event: &MouseMoved, args: ui::EventArgs) {
        let event = DragInputEvent::MouseMoved(event.0);
        args.event_queue.push(Target::Ui, event);
    }
}
pub struct DragMouseReleaseHandler;
impl ui::EventHandler<MouseButton> for DragMouseReleaseHandler {
    fn handle(&mut self, event: &MouseButton, args: ui::EventArgs) {
        let &MouseButton(state, _) = event;
        match state {
            glutin::ElementState::Released => {
                args.event_queue.push(Target::Ui, DragInputEvent::MouseReleased);
            }
            _ => (),
        }
    }
}
