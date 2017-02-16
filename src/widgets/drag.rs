use glutin;

use widget::{EventArgs, EventHandler};
use event::EventAddress;
use event::events::*;
use util::Point;

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
    dragging: bool,
    position: Point,
}
impl DragInputHandler {
    pub fn new() -> Self {
        DragInputHandler {
            dragging: false,
            position: Point::new(0.0, 0.0),
        }
    }
}
impl EventHandler<DragInputEvent> for DragInputHandler {
    fn handle(&mut self, event: &DragInputEvent, args: EventArgs) {
        match *event {
            DragInputEvent::WidgetPressed => {
                self.dragging = true;
                let event = WidgetDrag {
                    drag_type: DragEvent::DragStart,
                    position: self.position,
                };
                args.event_queue.push(EventAddress::Widget(args.widget_id), event);
            }
            DragInputEvent::MouseReleased => {
                if self.dragging {
                    self.dragging = false;
                    let event = WidgetDrag {
                        drag_type: DragEvent::DragEnd,
                        position: self.position,
                    };
                    args.event_queue.push(EventAddress::Widget(args.widget_id), event);
                }
            }
            DragInputEvent::MouseMoved(point) => {
                self.position = point;
                if self.dragging {
                    let event = WidgetDrag {
                        drag_type: DragEvent::Drag,
                        position: self.position,
                    };
                    args.event_queue.push(EventAddress::Widget(args.widget_id), event);
                }
            }
        }
    }
}

pub enum DragInputEvent {
    WidgetPressed,
    MouseReleased,
    MouseMoved(Point),
}

pub struct DragWidgetPressHandler {}
impl EventHandler<WidgetMouseButton> for DragWidgetPressHandler {
    fn handle(&mut self, event: &WidgetMouseButton, args: EventArgs) {
        let &WidgetMouseButton(state, _) = event;
        match state {
            glutin::ElementState::Pressed => {
                args.event_queue.push(EventAddress::Widget(args.widget_id),
                                      DragInputEvent::WidgetPressed);
            }
            _ => (),
        }
    }
}
pub struct DragMouseCursorHandler {}
impl EventHandler<MouseMoved> for DragMouseCursorHandler {
    fn handle(&mut self, event: &MouseMoved, args: EventArgs) {
        args.event_queue.push(EventAddress::Widget(args.widget_id),
                              DragInputEvent::MouseMoved(event.0));
    }
}
pub struct DragMouseReleaseHandler {}
impl EventHandler<MouseButton> for DragMouseReleaseHandler {
    fn handle(&mut self, event: &MouseButton, args: EventArgs) {
        let &MouseButton(state, _) = event;
        match state {
            glutin::ElementState::Released => {
                args.event_queue.push(EventAddress::Widget(args.widget_id),
                                      DragInputEvent::MouseReleased);
            }
            _ => (),
        }
    }
}
