use std::collections::{HashSet, HashMap};
use std::f64;
use std::any::Any;

use petgraph::stable_graph::StableGraph;
use petgraph::graph::NodeIndex;
use petgraph::visit::Dfs;
use petgraph::{Direction, Directed};
use petgraph::stable_graph::Neighbors;

use glutin;

use cassowary::{Solver, Constraint};
use cassowary::strength::*;

use graphics::Context;

use backend::gfx::G2d;
use backend::glyph::GlyphCache;
use backend::window::Window;

use widget::Widget;
use widget::builder::WidgetBuilder;
use widget::layout::{self, LayoutBuilder, WidgetConstraint};
use event::{self, EventId, EventQueue, EventAddress, Hover, WIDGET_REDRAW, WIDGET_HOVER,
            WIDGET_SCROLL, WIDGET_PRESS};
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
        InputState {
            mouse: Point { x: 0.0, y: 0.0 },
            last_over: HashSet::new(),
        }
    }
}

// special event handler for access to Ui
pub trait UiEventHandler {
    fn event_id(&self) -> EventId;
    fn handle_event(&mut self, args: UiEventArgs);
}
pub struct UiEventArgs<'a> {
    pub data: &'a (Any + 'static),
    pub ui: &'a mut Ui,
}

pub struct RedrawHandler {}
impl UiEventHandler for RedrawHandler {
    fn event_id(&self) -> EventId {
        WIDGET_REDRAW
    }
    fn handle_event(&mut self, args: UiEventArgs) {
        let ui = args.ui;
        ui.dirty_widgets.insert(ui.root_index.unwrap());
    }
}

pub fn get_default_event_handlers() -> Vec<Box<UiEventHandler>> {
    vec!{Box::new(RedrawHandler{})}
}

pub struct Ui {
    pub graph: StableGraph<Widget, ()>,
    pub root_index: Option<NodeIndex>,
    pub solver: Solver,
    pub input_state: InputState,
    pub widget_map: HashMap<Id, NodeIndex>,
    pub constraint_map: HashMap<Id, Vec<Constraint>>,
    pub dirty_widgets: HashSet<NodeIndex>,
    pub glyph_cache: GlyphCache,
}
impl Ui {
    pub fn new(window: &mut Window) -> Self {
        Ui {
            graph: StableGraph::<Widget, ()>::new(),
            root_index: None,
            solver: Solver::new(),
            input_state: InputState::new(),
            widget_map: HashMap::new(),
            constraint_map: HashMap::new(),
            dirty_widgets: HashSet::new(),
            glyph_cache: GlyphCache::new(&mut window.context.factory, 512, 512),
        }
    }
    pub fn resize_window_to_fit(&mut self, window: &Window) {
        let window_dims = self.get_root_dims();
        window.window.set_inner_size(window_dims.width as u32, window_dims.height as u32);
    }
    pub fn set_root(&mut self, mut root_widget: WidgetBuilder) {
        root_widget.layout.top_left(Point { x: 0.0, y: 0.0 }, None);
        self.root_index = Some(self.add_widget(root_widget, None));
        let ref mut root = &mut self.graph[self.root_index.unwrap()];
        self.solver.add_edit_variable(root.layout.right, STRONG).unwrap();
        self.solver.add_edit_variable(root.layout.bottom, STRONG).unwrap();
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
        self.dirty_widgets.insert(self.root_index.unwrap());
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
                self.draw_node(context, graphics, child_index, crop_to);
            }
        }
    }
    pub fn draw(&mut self, context: Context, graphics: &mut G2d) {

        let index = self.root_index.unwrap().clone();
        let crop_to = Rectangle {
            top: 0.0,
            left: 0.0,
            width: f64::MAX,
            height: f64::MAX,
        };
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
    pub fn add_widget(&mut self, mut widget: WidgetBuilder, parent_index: Option<NodeIndex>) -> NodeIndex {

        let (children, constraints, widget) = widget.build();
        self.constraint_map.insert(widget.id, Vec::new());
        for constraint in constraints {
            // insert constraint into list for both widgets it affects,
            // so that if either widget is removed, the constraint is as well
            let constraint = match constraint {
                WidgetConstraint::Local(constraint) => constraint,
                WidgetConstraint::Relative(constraint, widget_id) => {
                    if !self.constraint_map.contains_key(&widget_id) {
                        self.constraint_map.insert(widget_id, Vec::new());
                    }
                    if let Some(constraint_list) = self.constraint_map.get_mut(&widget.id) {
                        constraint_list.push(constraint.clone());
                    }
                    constraint
                }
            };
            if let Some(constraint_list) = self.constraint_map.get_mut(&widget.id) {
                constraint_list.push(constraint.clone());
            }
            self.solver.add_constraint(constraint).unwrap();
        }
        let id = widget.id;
        let widget_index = self.graph.add_node(widget);
        if let Some(parent_index) = parent_index {
            self.graph.add_edge(parent_index, widget_index, ());
        }
        self.widget_map.insert(id, widget_index);
        self.dirty_widgets.insert(widget_index);
        for child in children {
            self.add_widget(*child, Some(widget_index));
        }
        widget_index
    }

    pub fn remove_widget(&mut self, widget_id: Id) {
        if let Some(node_index) = self.find_widget(widget_id) {
            self.graph.remove_node(node_index);
            self.dirty_widgets.insert(self.root_index.unwrap());
            // remove constraints that are relative to this widget from solver
            if let Some(constraint_list) = self.constraint_map.get(&widget_id) {
                for constraint in constraint_list {
                    if self.solver.has_constraint(constraint) {
                        self.solver.remove_constraint(constraint);
                    }
                }
            }
            // doesn't clean up other references to these constraints in the constraint map, but at least they won't affect the solver
            self.constraint_map.remove(&widget_id);
        }
    }
    pub fn get_widget(&self, widget_id: Id) -> Option<&Widget> {
        self.widget_map.get(&widget_id).and_then(|node_index| {
            let ref widget = self.graph[NodeIndex::new(node_index.index())];
            return Some(widget);
        });
        None
    }
    pub fn handle_event(&mut self, event: glutin::Event, event_queue: &mut EventQueue) {
        match event {
            glutin::Event::MouseMoved(x, y) => {
                let mouse = Point {
                    x: x as f64,
                    y: y as f64,
                };
                self.input_state.mouse = mouse;
                let last_over = self.input_state.last_over.clone();
                for last_over in last_over {
                    let last_over = last_over.clone();
                    if let Some(last_index) = self.find_widget(last_over) {
                        let ref mut widget = self.graph[last_index];
                        if !widget.is_mouse_over(&mut self.solver, self.input_state.mouse) {
                            event_queue.push(EventAddress::Widget(last_over),
                                             WIDGET_HOVER,
                                             Box::new(Hover::Out));
                            self.input_state.last_over.remove(&last_over);
                        }
                    }
                }
                event_queue.push(EventAddress::UnderMouse,
                                 WIDGET_HOVER,
                                 Box::new(Hover::Over));
            }
            _ => (),
        }
        if let Some(event_id) = mouse_under_event(&event) {
            event_queue.push(EventAddress::UnderMouse, event_id, Box::new(event));
        }
    }

    pub fn check_layout(&mut self, event_queue: &mut EventQueue) {
        // if layout has changed, send new mouse event, in case widget under mouse has shifted
        let has_changes = self.solver.fetch_changes().len() > 0;
        if has_changes {
            //self.solver.debug_constraints();
            let mouse = self.input_state.mouse;
            let event = glutin::Event::MouseMoved(mouse.x as i32, mouse.y as i32);
            self.handle_event(event, event_queue);
        }
    }
    pub fn is_mouse_over(&mut self, node_index: NodeIndex) -> bool {
        let ref mut widget = self.graph[node_index];
        widget.is_mouse_over(&mut self.solver, self.input_state.mouse)
    }
    pub fn find_widget(&mut self, widget_id: Id) -> Option<NodeIndex> {
        self.widget_map.get(&widget_id).map(|index| *index)
    }

    pub fn trigger_widget_event(&mut self,
                            node_index: NodeIndex,
                            event_id: EventId,
                            data: &(Any + 'static),
                            event_queue: &mut EventQueue) -> bool {
        let ref mut widget = self.graph[node_index];
        let handled = widget.trigger_event(event_id, data, event_queue, &mut self.solver, &self.input_state);
        if let Some(ref mut drawable) = widget.drawable {
            if drawable.has_updated {
                self.dirty_widgets.insert(node_index);
                drawable.has_updated = false;
            }
        }
        handled
    }
}

// get the widget event that is received if the event occurs while mouse is over widget
pub fn mouse_under_event(event: &glutin::Event) -> Option<EventId> {
    match *event {
        glutin::Event::MouseWheel(..) => Some(WIDGET_SCROLL),
        glutin::Event::MouseInput(..) => Some(WIDGET_PRESS),
        _ => None,
    }
}
