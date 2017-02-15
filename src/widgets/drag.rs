use glutin;

use widget::{EventArgs, EventHandler};
use event::{EventId, EventAddress};
use event::events::*;
use event::id::*;

pub struct WidgetDrag(pub (DragEvent, (i32, i32)));
pub struct DragInput(pub glutin::Event);

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
    fn handle(&mut self, args: EventArgs<DragInputEvent>) {
        let event = args.event;
        match *event {
            DragInputEvent::WidgetPressed => {
                self.dragging = true;
                let event = WidgetDrag((DragEvent::DragStart, self.position));
                args.event_queue.push(EventAddress::Widget(args.widget_id), NONE, event);
            }
            DragInputEvent::MouseReleased => {
                if self.dragging {
                    self.dragging = false;
                    let event = WidgetDrag((DragEvent::DragEnd, self.position));
                    args.event_queue.push(EventAddress::Widget(args.widget_id), NONE, event);
                }
            }
            DragInputEvent::MouseMoved(ref event) => {
                match *event {
                    glutin::Event::MouseMoved(x, y) => {
                        self.position = (x, y);
                        if self.dragging {
                            let event = WidgetDrag((DragEvent::Drag, self.position));
                            args.event_queue.push(EventAddress::Widget(args.widget_id), NONE, event);
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
    fn handle(&mut self, args: EventArgs<WidgetMouseButton>) {
        let ref event = args.event.0;
        match *event {
            glutin::Event::MouseInput(state, _) => {
                match state {
                    glutin::ElementState::Pressed => {
                        args.event_queue.push(EventAddress::Widget(args.widget_id), NONE, DragInputEvent::WidgetPressed);
                    }, _ => ()
                }
            }, _ => ()
        }
    }
}
pub struct DragMouseCursorHandler {}
impl EventHandler<MouseMoved> for DragMouseCursorHandler {
    fn handle(&mut self, args: EventArgs<MouseMoved>) {
        let ref event = args.event.0;
        args.event_queue.push(EventAddress::Widget(args.widget_id), NONE, DragInputEvent::MouseMoved(event.clone()));
    }
}
pub struct DragMouseReleaseHandler {}
impl EventHandler<MouseButton> for DragMouseReleaseHandler {
    fn handle(&mut self, args: EventArgs<MouseButton>) {
        let ref event = args.event.0;
        match *event {
            glutin::Event::MouseInput(state, _) => {
                match state {
                    glutin::ElementState::Released => {
                        args.event_queue.push(EventAddress::Widget(args.widget_id), NONE, DragInputEvent::MouseReleased);
                    }, _ => ()
                }
            }, _ => ()
        }
    }
}