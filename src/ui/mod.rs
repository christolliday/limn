pub mod graph;
pub mod queue;
pub mod event;
pub mod layout;

pub use self::graph::WidgetGraph;
pub use self::queue::{EventQueue, EventAddress};
pub use self::event::*;
pub use self::layout::LimnSolver;

use backend::Window;

use std::any::{Any, TypeId};
use std::collections::HashSet;

use glutin;

use util::Point;
use resources::WidgetId;

use widgets::hover::Hover;

pub struct Ui {
    pub graph: WidgetGraph,
    pub solver: LimnSolver,
    pub input_state: InputState,
}

impl Ui {
    pub fn new(window: &mut Window, event_queue: &EventQueue) -> Self {
        let graph = WidgetGraph::new(window);
        let solver = LimnSolver::new(event_queue.clone());
        Ui {
            graph: graph,
            solver: solver,
            input_state: InputState::new(),
        }
    }
}
pub fn handle_input(event: glutin::Event, args: EventArgs) {
    let EventArgs { graph, event_queue, input_state, .. } = args;
    match event {
        glutin::Event::MouseMoved(x, y) => {
            let mouse = Point {
                x: x as f64,
                y: y as f64,
            };
            input_state.mouse = mouse;
            let last_over = input_state.last_over.clone();
            for last_over in last_over {
                let last_over = last_over.clone();
                if let Some(last_index) = graph.find_widget(last_over) {
                    if let Some(widget) = graph.get_widget_index(last_index) {
                        if !widget.is_mouse_over(input_state.mouse) {
                            event_queue.push(EventAddress::Widget(last_over), Hover::Out);
                            input_state.last_over.remove(&last_over);
                        }
                    }
                }
            }
            event_queue.push(EventAddress::UnderMouse, Hover::Over);
        }
        _ => (),
    }
    let ref root_widget = graph.get_root();
    let all_widgets = EventAddress::SubTree(root_widget.id);
    match event {
        glutin::Event::MouseWheel(mouse_scroll_delta, _) => {
            event_queue.push(EventAddress::UnderMouse,
                             WidgetMouseWheel(mouse_scroll_delta));
            event_queue.push(all_widgets, MouseWheel(mouse_scroll_delta));
        }
        glutin::Event::MouseInput(state, button) => {
            event_queue.push(EventAddress::UnderMouse, WidgetMouseButton(state, button));
            event_queue.push(all_widgets, MouseButton(state, button));
        }
        glutin::Event::MouseMoved(x, y) => {
            event_queue.push(all_widgets, MouseMoved(Point::new(x as f64, y as f64)));
        }
        _ => (),
    }
}

pub struct EventArgs<'a> {
    pub graph: &'a mut WidgetGraph,
    pub event_queue: &'a mut EventQueue,
    pub input_state: &'a mut InputState,
    pub solver:&'a mut LimnSolver,
}

pub trait EventHandler<T> {
    fn handle(&mut self, event: &T, args: EventArgs);
}

pub struct HandlerWrapper {
    type_id: TypeId,
    handler: Box<Any>,
    handle_fn: Box<Fn(&mut Box<Any>, &Box<Any + Send>, EventArgs)>,
}
impl HandlerWrapper {
    pub fn new<H, E>(handler: H) -> Self
        where H: EventHandler<E> + 'static,
              E: 'static
    {
        let handle_fn = |handler: &mut Box<Any>, event: &Box<Any + Send>, args: EventArgs| {
            let event: &E = event.downcast_ref().unwrap();
            let handler: &mut H = handler.downcast_mut().unwrap();
            handler.handle(event, args);
        };
        HandlerWrapper {
            type_id: TypeId::of::<E>(),
            handler: Box::new(handler),
            handle_fn: Box::new(handle_fn),
        }
    }
    pub fn handles(&self, type_id: TypeId) -> bool {
        self.type_id == type_id
    }
    pub fn handle(&mut self, event: &Box<Any + Send>, args: EventArgs) {
        (self.handle_fn)(&mut self.handler, event, args);
    }
}


pub struct InputEvent(pub glutin::Event);
pub struct RedrawEvent;
pub struct LayoutChanged(pub WidgetId);

pub struct InputState {
    pub mouse: Point,
    pub last_over: HashSet<WidgetId>,
}
impl InputState {
    fn new() -> Self {
        InputState {
            mouse: Point { x: 0.0, y: 0.0 },
            last_over: HashSet::new(),
        }
    }
}

pub struct InputHandler;
impl EventHandler<InputEvent> for InputHandler {
    fn handle(&mut self, event: &InputEvent, args: EventArgs) {
        handle_input(event.0.clone(), args);
    }
}

pub struct RedrawHandler;
impl EventHandler<RedrawEvent> for RedrawHandler {
    fn handle(&mut self, _: &RedrawEvent, args: EventArgs) {
        let graph = args.graph;
        graph.dirty_widgets.insert(graph.root_index.unwrap());
    }
}
pub struct LayoutChangeHandler;
impl EventHandler<LayoutChanged> for LayoutChangeHandler {
    fn handle(&mut self, event: &LayoutChanged, args: EventArgs) {
        let graph = args.graph;
        {
            let &LayoutChanged(widget_id) = event;
            if let Some(widget) = graph.get_widget(widget_id) {
                widget.layout.update(args.solver);
            }
        }
        // redraw everything when layout changes, for now
        args.event_queue.push(EventAddress::Ui, RedrawEvent);
        // send new mouse event, in case widget under mouse has shifted
        let mouse = args.input_state.mouse;
        let event = glutin::Event::MouseMoved(mouse.x as i32, mouse.y as i32);
        args.event_queue.push(EventAddress::Ui, InputEvent(event));
    }
}

pub fn get_default_event_handlers() -> Vec<HandlerWrapper> {
    vec![
        HandlerWrapper::new(RedrawHandler),
        HandlerWrapper::new(LayoutChangeHandler),
        HandlerWrapper::new(InputHandler),
    ]
}