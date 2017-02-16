use glutin;

use widget::{EventArgs, EventHandler};
use event::EventAddress;
use event::events::*;

pub struct WidgetDrag {
    pub drag_type: DragEvent,
    pub position: (i32, i32),
}

#[derive(Debug)]
pub enum DragEvent {
    DragStart,
    Drag,
    DragEnd,
}

pub struct DragInputHandler {
    dragging: bool,
    position: (i32, i32),
}
impl DragInputHandler {
    pub fn new() -> Self {
        DragInputHandler { dragging: false, position: (0, 0) }
    }
}
impl EventHandler<DragInputEvent> for DragInputHandler {
    fn handle(&mut self, event: &DragInputEvent, args: EventArgs) {
        match *event {
            DragInputEvent::WidgetPressed => {
                self.dragging = true;
                let event = WidgetDrag { drag_type: DragEvent::DragStart, position: self.position };
                args.event_queue.push(EventAddress::Widget(args.widget_id), event);
            }
            DragInputEvent::MouseReleased => {
                if self.dragging {
                    self.dragging = false;
                    let event = WidgetDrag { drag_type: DragEvent::DragEnd, position: self.position };
                    args.event_queue.push(EventAddress::Widget(args.widget_id), event);
                }
            }
            DragInputEvent::MouseMoved(ref event) => {
                match *event {
                    glutin::Event::MouseMoved(x, y) => {
                        self.position = (x, y);
                        if self.dragging {
                            let event = WidgetDrag { drag_type: DragEvent::Drag, position: self.position };
                            args.event_queue.push(EventAddress::Widget(args.widget_id), event);
                        }
                    }, _ => ()
                }
            }
        }
    }
}

pub enum DragInputEvent {
    WidgetPressed,
    MouseReleased,
    MouseMoved(glutin::Event),
}

pub struct DragWidgetPressHandler {}
impl EventHandler<WidgetMouseButton> for DragWidgetPressHandler {
    fn handle(&mut self, event: &WidgetMouseButton, args: EventArgs) {
        let ref event = event.0;
        match *event {
            glutin::Event::MouseInput(state, _) => {
                match state {
                    glutin::ElementState::Pressed => {
                        args.event_queue.push(EventAddress::Widget(args.widget_id), DragInputEvent::WidgetPressed);
                    }, _ => ()
                }
            }, _ => ()
        }
    }
}
pub struct DragMouseCursorHandler {}
impl EventHandler<MouseMoved> for DragMouseCursorHandler {
    fn handle(&mut self, event: &MouseMoved, args: EventArgs) {
        let ref event = event.0;
        args.event_queue.push(EventAddress::Widget(args.widget_id), DragInputEvent::MouseMoved(event.clone()));
    }
}
pub struct DragMouseReleaseHandler {}
impl EventHandler<MouseButton> for DragMouseReleaseHandler {
    fn handle(&mut self, event: &MouseButton, args: EventArgs) {
        let ref event = event.0;
        match *event {
            glutin::Event::MouseInput(state, _) => {
                match state {
                    glutin::ElementState::Released => {
                        args.event_queue.push(EventAddress::Widget(args.widget_id), DragInputEvent::MouseReleased);
                    }, _ => ()
                }
            }, _ => ()
        }
    }
}