use glutin;

use widget::{EventArgs, EventHandler};
use event::{EventId, EventAddress};
use event::id::*;

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
impl EventHandler for DragInputHandler {
    fn event_id(&self) -> EventId {
        DRAG_INPUT_EVENT
    }
    fn handle_event(&mut self, args: EventArgs) {
        let event = args.data.downcast_ref::<DragInputEvent>().unwrap();
        match *event {
            DragInputEvent::WidgetPressed => {
                self.dragging = true;
                let event = (DragEvent::DragStart, self.position);
                args.event_queue.push(EventAddress::Widget(args.widget_id), WIDGET_DRAG, event);
            }
            DragInputEvent::MouseReleased => {
                if self.dragging {
                    self.dragging = false;
                    let event = (DragEvent::DragEnd, self.position);
                    args.event_queue.push(EventAddress::Widget(args.widget_id), WIDGET_DRAG, event);
                }
            }
            DragInputEvent::MouseMoved(ref event) => {
                match *event {
                    glutin::Event::MouseMoved(x, y) => {
                        self.position = (x, y);
                        if self.dragging {
                            let event = (DragEvent::Drag, self.position);
                            args.event_queue.push(EventAddress::Widget(args.widget_id), WIDGET_DRAG, event);
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
impl EventHandler for DragWidgetPressHandler {
    fn event_id(&self) -> EventId {
        WIDGET_MOUSE_BUTTON
    }
    fn handle_event(&mut self, args: EventArgs) {
        let event = args.data.downcast_ref::<glutin::Event>().unwrap();
        match *event {
            glutin::Event::MouseInput(state, button) => {
                match state {
                    glutin::ElementState::Pressed => {
                        args.event_queue.push(EventAddress::Widget(args.widget_id), DRAG_INPUT_EVENT, DragInputEvent::WidgetPressed);
                    }, _ => ()
                }
            }, _ => ()
        }
    }
}
pub struct DragMouseCursorHandler {}
impl EventHandler for DragMouseCursorHandler {
    fn event_id(&self) -> EventId {
        MOUSE_MOVED
    }
    fn handle_event(&mut self, args: EventArgs) {
        let event = args.data.downcast_ref::<glutin::Event>().unwrap();
        args.event_queue.push(EventAddress::Widget(args.widget_id), DRAG_INPUT_EVENT, DragInputEvent::MouseMoved(event.clone()));
    }
}
pub struct DragMouseReleaseHandler {}
impl EventHandler for DragMouseReleaseHandler {
    fn event_id(&self) -> EventId {
        MOUSE_BUTTON
    }
    fn handle_event(&mut self, args: EventArgs) {
        let event = args.data.downcast_ref::<glutin::Event>().unwrap();
        match *event {
            glutin::Event::MouseInput(state, button) => {
                match state {
                    glutin::ElementState::Released => {
                        args.event_queue.push(EventAddress::Widget(args.widget_id), DRAG_INPUT_EVENT, DragInputEvent::MouseReleased);
                    }, _ => ()
                }
            }, _ => ()
        }
    }
}