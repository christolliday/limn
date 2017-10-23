use glutin;

use event::{EventHandler, EventArgs};
use widget::{WidgetBuilder, WidgetRef};
use input::mouse::{MouseMoved, MouseButton, WidgetMouseButton};
use geometry::{Point, Vector};
use app::App;

#[derive(Debug, Copy, Clone)]
pub struct DragEvent {
    pub state: DragState,
    /// mouse position
    pub position: Point,
    /// offset from drag start
    pub offset: Vector,
    /// change since last DragEvent
    pub change: Vector,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DragState {
    Start,
    Moved,
    End,
}

#[derive(Debug, Clone)]
struct DragInputHandler {
    widget: Option<WidgetRef>,
    position: Point,
    start_position: Point,
    prev_position: Point,
}

impl DragInputHandler {
    pub fn new() -> Self {
        DragInputHandler {
            widget: None,
            position: Point::new(0.0, 0.0),
            start_position: Point::new(0.0, 0.0),
            prev_position: Point::new(0.0, 0.0),
        }
    }
    pub fn drag_event(&self, state: DragState) -> DragEvent {
        DragEvent {
            state: state,
            position: self.position,
            offset: self.position - self.start_position,
            change: self.position - self.prev_position,
        }
    }
}

impl EventHandler<DragInputEvent> for DragInputHandler {
    fn handle(&mut self, event: &DragInputEvent, _: EventArgs) {
        match *event {
            DragInputEvent::WidgetPressed(ref widget) => {
                self.widget = Some(widget.clone());
                self.start_position = self.position;
                widget.event(self.drag_event(DragState::Start));
            }
            DragInputEvent::MouseMoved(point) => {
                self.prev_position = self.position;
                self.position = point;
                if let Some(ref widget) = self.widget {
                    widget.event(self.drag_event(DragState::Moved));
                }
            }
            DragInputEvent::MouseReleased => {
                if let Some(widget) = self.widget.take() {
                    widget.event(self.drag_event(DragState::End));
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
enum DragInputEvent {
    WidgetPressed(WidgetRef),
    MouseMoved(Point),
    MouseReleased,
}

impl WidgetBuilder {
    /// Make a widget receive drag events.
    pub fn make_draggable(&mut self) -> &mut Self {
        self.add_handler_fn(|event: &WidgetMouseButton, args| {
            if let WidgetMouseButton(glutin::ElementState::Pressed, _) = *event {
                let event = DragInputEvent::WidgetPressed(args.widget);
                args.ui.event(event);
            }
        });
        self
    }
}

impl App {
    /// Add handlers to UI to enable drag detection
    /// UI receives messages from draggable widgets when they are clicked,
    /// and combines that with mouse move and mouse release events to
    /// synthesize `DragEvent`s
    pub fn add_drag_handlers(&mut self) {
        self.add_handler(DragInputHandler::new());
        self.add_handler_fn(|event: &MouseMoved, args| {
            args.ui.event(DragInputEvent::MouseMoved(event.0));
        });
        self.add_handler_fn(|event: &MouseButton, args| {
            if let MouseButton(glutin::ElementState::Released, _) = *event {
                args.ui.event(DragInputEvent::MouseReleased);
            }
        });
    }
}
