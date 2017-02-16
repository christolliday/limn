pub mod graph;

pub use self::graph::WidgetGraph;

use backend::Window;

use std::any::{Any, TypeId};
use std::collections::HashSet;

use glutin;

use event::{EventQueue, EventAddress};
use util::Point;
use resources::WidgetId;

pub struct Ui {
    pub event_queue: EventQueue,
    pub graph: WidgetGraph,
}

impl Ui {
    pub fn new(window: &mut Window) -> Self {
        let event_queue = EventQueue::new(window);
        let graph = WidgetGraph::new(window, &event_queue);
        Ui {
            event_queue: event_queue,
            graph: graph,
        }
    }
}

pub struct EventArgs<'a> {
    pub graph: &'a mut WidgetGraph,
    pub event_queue: &'a mut EventQueue,
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


pub struct Redraw(());
pub struct Layout(pub WidgetId);

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

pub struct RedrawHandler {}
impl EventHandler<Redraw> for RedrawHandler {
    fn handle(&mut self, _: &Redraw, args: EventArgs) {
        let graph = args.graph;
        graph.dirty_widgets.insert(graph.root_index.unwrap());
    }
}
pub struct LayoutHandler {}
impl EventHandler<Layout> for LayoutHandler {
    fn handle(&mut self, event: &Layout, args: EventArgs) {
        let graph = args.graph;
        {
            let &Layout(widget_id) = event;
            let node_index = graph.find_widget(widget_id).unwrap();
            let ref mut widget = graph.graph[node_index];
            widget.layout.update(&mut graph.solver);
        }
        // redraw everything when layout changes, for now
        args.event_queue.push(EventAddress::Ui, Redraw(()));
        // send new mouse event, in case widget under mouse has shifted
        let mouse = graph.input_state.mouse;
        let event = glutin::Event::MouseMoved(mouse.x as i32, mouse.y as i32);
        graph.handle_input(event, args.event_queue);
    }
}

pub fn get_default_event_handlers() -> Vec<HandlerWrapper> {
    vec![HandlerWrapper::new(RedrawHandler {}), HandlerWrapper::new(LayoutHandler {})]
}