use std::collections::{HashSet, HashMap};
use std::f64;

use petgraph::Graph;
use petgraph::graph::NodeIndex;
use petgraph::visit::Dfs;
use petgraph::Direction;
use petgraph::graph::Neighbors;

use input;
use input::{GenericEvent, MouseCursorEvent};

use cassowary::Solver;
use cassowary::strength::*;

use graphics::Context;

use backend::gfx::G2d;
use backend::glyph::GlyphCache;
use backend::window::Window;

use widget::Widget;
use widget::builder::WidgetBuilder;
use event::{self, Event, Signal, InputEvent, EventQueue, EventAddress};
use util::{self, Point, Rectangle, Dimensions};
use resources::Id;
use color::*;

const DEBUG_BOUNDS: bool = false;

pub struct InputState {
    pub mouse: Point,
    pub last_over: HashSet<Id>,
}
impl InputState {
    fn new() -> Self {
        InputState { mouse: Point { x: 0.0, y: 0.0 }, last_over: HashSet::new() }
    }
}

pub struct Ui {
    pub graph: Graph<Widget, ()>,
    pub root_index: Option<NodeIndex>,
    pub solver: Solver,
    pub input_state: InputState,
    pub widget_map: HashMap<Id, NodeIndex>,
    pub dirty_widgets: HashSet<NodeIndex>,
    pub event_queue: EventQueue,
    pub glyph_cache: GlyphCache,
}
impl Ui {
    pub fn new(window: &mut Window) -> Self {
        Ui {
            graph: Graph::<Widget, ()>::new(),
            root_index: None,
            solver: Solver::new(),
            input_state: InputState::new(),
            widget_map: HashMap::new(),
            dirty_widgets: HashSet::new(),
            event_queue: EventQueue::new(window),
            glyph_cache: GlyphCache::new(&mut window.context.factory, 512, 512),
        }
    }
    pub fn resize_window_to_fit(&mut self, window: &Window) {
        let window_dims = self.get_root_dims();
        window.window.window.set_inner_size(window_dims.width as u32, window_dims.height as u32);
    }
    pub fn set_root(&mut self, root_widget: WidgetBuilder) {
        self.root_index = Some(root_widget.create(self, None));
        let ref mut root = &mut self.graph[self.root_index.unwrap()];
        self.solver.add_edit_variable(root.layout.right, STRONG).unwrap();
        self.solver.add_edit_variable(root.layout.bottom, STRONG).unwrap();
        root.layout.top_left(Point { x: 0.0, y: 0.0 });
        root.layout.update_solver(&mut self.solver);
    }
    pub fn get_root(&mut self) -> &Widget {
        &self.graph[self.root_index.unwrap()]
    }
    pub fn get_root_dims(&mut self) -> Dimensions {
        let ref mut root = &mut self.graph[self.root_index.unwrap()];
        root.layout.get_dims(&mut self.solver)
    }
    pub fn window_resized(&mut self, window_dims: Dimensions) {
        let ref root = self.graph[self.root_index.unwrap()];
        self.solver.suggest_value(root.layout.right, window_dims.width).unwrap();
        self.solver.suggest_value(root.layout.bottom, window_dims.height).unwrap();
    }
    pub fn parents(&mut self, node_index: NodeIndex) -> Neighbors<()> {
        self.graph.neighbors_directed(node_index, Direction::Incoming)
    }
    pub fn children(&mut self, node_index: NodeIndex) -> Neighbors<()> {
        self.graph.neighbors_directed(node_index, Direction::Outgoing)
    }

    pub fn draw_node(&mut self,
                     context: Context,
                     graphics: &mut G2d,
                     node_index: NodeIndex,
                     crop_to: Rectangle) {

        let crop_to = {
            let ref mut widget = self.graph[node_index];
            widget.draw(crop_to,
                        &mut self.solver,
                        &mut self.glyph_cache,
                        context,
                        graphics);

            util::crop_rect(crop_to, widget.layout.bounds(&mut self.solver))
        };

        if !crop_to.no_area() {
            let children: Vec<NodeIndex> = self.children(node_index).collect();
            // need to iterate backwards to draw in correct order, because 
            // petgraph neighbours iterate in reverse order of insertion, not sure why
            for child_index in children.iter().rev() {
                let child_index = child_index.clone();
                self.draw_node(context,
                            graphics,
                            child_index,
                            crop_to);
            }
        }
    }
    pub fn draw(&mut self,
                context: Context,
                graphics: &mut G2d) {

        let index = self.root_index.unwrap().clone();
        let crop_to = Rectangle { top: 0.0, left: 0.0, width: f64::MAX, height: f64::MAX };
        self.draw_node(context, graphics, index, crop_to);

        if DEBUG_BOUNDS {
            let mut dfs = Dfs::new(&self.graph, self.root_index.unwrap());
            while let Some(node_index) = dfs.next(&self.graph) {
                let ref widget = self.graph[node_index];
                let color = widget.debug_color.unwrap_or(GREEN);
                let bounds = widget.layout.bounds(&mut self.solver);
                util::draw_rect_outline(bounds, color, context, graphics);
            }
        }
    }
    pub fn add_widget(&mut self, parent_index: Option<NodeIndex>, child: Widget) -> NodeIndex {
        let id = child.id;
        let child_index = self.graph.add_node(child);
        if let Some(parent_index) = parent_index {
            self.graph.add_edge(parent_index, child_index, ());
        }
        self.widget_map.insert(id, child_index);
        self.dirty_widgets.insert(child_index);
        child_index
    }
    pub fn get_widget(&self, widget_id: Id) -> Option<&Widget> {
        self.widget_map.get(&widget_id).and_then(|node_index| {
            let ref widget = self.graph[NodeIndex::new(node_index.index())];
            return Some(widget);
        });
        None
    }
    pub fn handle_event(&mut self, event: input::Event) {
        if let Some(mouse) = event.mouse_cursor_args() {
            self.input_state.mouse = mouse.into();
            let last_over = self.input_state.last_over.clone();
            for last_over in last_over {
                let last_over = last_over.clone();
                if let Some(last_index) = self.find_widget(last_over) {
                    let ref mut widget = self.graph[last_index];
                    if !widget.is_mouse_over(&mut self.solver, self.input_state.mouse) {
                        let event = Signal::new(event::WIDGET_MOUSE_OFF);
                        self.event_queue.push(EventAddress::Widget(last_over), Box::new(event));
                        self.input_state.last_over.remove(&last_over);
                    }
                }
            }
        }
        if let Some(event_id) = event::widget_event(event.event_id()) {
            let event = InputEvent::new(event_id, event);
            self.event_queue.push(EventAddress::UnderMouse, Box::new(event));
        }
    }
    pub fn handle_event_queue(&mut self) {
        while !self.event_queue.is_empty() {
            let (event_address, event) = self.event_queue.next();
            let event = &*event;
            match event_address {
                EventAddress::Widget(id) => {
                    if let Some(node_index) = self.find_widget(id) {
                        self.trigger_widget_event(node_index, event);
                    }
                },
                EventAddress::Child(id) => {
                    if let Some(node_index) = self.find_widget(id) {
                        if let Some(child_index) = self.children(node_index).next() {
                            self.trigger_widget_event(child_index, event);
                        }
                    }
                }
                EventAddress::SubTree(id) => {
                    if let Some(node_index) = self.find_widget(id) {
                        let mut dfs = Dfs::new(&self.graph, node_index);
                        while let Some(node_index) = dfs.next(&self.graph) {
                            self.trigger_widget_event(node_index, event);
                        }
                    }
                },
                EventAddress::UnderMouse => {
                    let mut dfs = Dfs::new(&self.graph, self.root_index.unwrap());
                    while let Some(node_index) = dfs.next(&self.graph) {
                        let is_mouse_over = self.is_mouse_over(node_index);
                        if is_mouse_over {
                            self.trigger_widget_event(node_index, event);
                            let ref mut widget = self.graph[node_index];
                            self.input_state.last_over.insert(widget.id);
                        }
                    }
                }
            }
        }
    }
    fn is_mouse_over(&mut self, node_index: NodeIndex) -> bool {
        let ref mut widget = self.graph[node_index];
        widget.is_mouse_over(&mut self.solver, self.input_state.mouse)
    }
    fn find_widget(&mut self, widget_id: Id) -> Option<NodeIndex> {
        self.widget_map.get(&widget_id).map(|index| *index)
    }

    fn trigger_widget_event(&mut self, node_index: NodeIndex, event: &(Event + 'static)) {
        let ref mut widget = self.graph[node_index];
        widget.trigger_event(event, &mut self.event_queue, &mut self.solver);
        if widget.drawable.has_updated {
            self.dirty_widgets.insert(node_index);
            widget.drawable.has_updated = false;
        }
    }
}
