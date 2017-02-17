use std::collections::{HashSet, HashMap};
use std::f64;
use std::any::{Any, TypeId};

use petgraph::stable_graph::StableGraph;
use petgraph::graph::NodeIndex;
use petgraph::visit::{Dfs, DfsPostOrder};
use petgraph::Direction;
use petgraph::stable_graph::Neighbors;

use cassowary::strength::*;

use graphics::Context;

use backend::gfx::G2d;
use backend::glyph::GlyphCache;
use backend::window::Window;

use widget::Widget;
use widget::builder::WidgetBuilder;
use util::{self, Point, Rectangle, Dimensions};
use resources::WidgetId;
use color::*;

use super::layout::LimnSolver;
use super::queue::EventQueue;
use super::InputState;

const DEBUG_BOUNDS: bool = false;

pub struct WidgetGraph {
    graph: StableGraph<Widget, ()>,
    pub root_index: Option<NodeIndex>,
    pub widget_map: HashMap<WidgetId, NodeIndex>,
    pub dirty_widgets: HashSet<NodeIndex>,
    pub glyph_cache: GlyphCache,
}

impl WidgetGraph {
    pub fn new(window: &mut Window) -> Self {
        WidgetGraph {
            graph: StableGraph::<Widget, ()>::new(),
            root_index: None,
            widget_map: HashMap::new(),
            dirty_widgets: HashSet::new(),
            glyph_cache: GlyphCache::new(&mut window.context.factory, 512, 512),
        }
    }
    pub fn resize_window_to_fit(&mut self, window: &Window, solver: &mut LimnSolver) {
        let window_dims = self.get_root_dims(solver);
        window.window.set_inner_size(window_dims.width as u32, window_dims.height as u32);
    }
    pub fn set_root(&mut self, mut root_widget: WidgetBuilder, solver: &mut LimnSolver) {
        root_widget.layout.top_left(Point { x: 0.0, y: 0.0 }, None);
        self.root_index = Some(self.add_widget(root_widget, None, solver));
        let ref mut root = &mut self.graph[self.root_index.unwrap()];
        solver.update_solver(|solver| {
            solver.add_edit_variable(root.layout.right, STRONG).unwrap();
            solver.add_edit_variable(root.layout.bottom, STRONG).unwrap();
        });
    }
    pub fn get_root(&mut self) -> &Widget {
        &self.graph[self.root_index.unwrap()]
    }
    pub fn get_root_dims(&mut self, solver: &mut LimnSolver) -> Dimensions {
        let ref mut root = &mut self.graph[self.root_index.unwrap()];
        root.layout.update(solver);
        root.layout.get_dims()
    }
    pub fn window_resized(&mut self, window_dims: Dimensions, solver: &mut LimnSolver) {
        let ref mut root = self.graph[self.root_index.unwrap()];
        root.layout.update(solver);
        solver.update_solver(|solver| {
            solver.suggest_value(root.layout.right, window_dims.width).unwrap();
            solver.suggest_value(root.layout.bottom, window_dims.height).unwrap();
        });
        self.dirty_widgets.insert(self.root_index.unwrap());
    }
    pub fn parents(&mut self, node_index: NodeIndex) -> Neighbors<()> {
        self.graph.neighbors_directed(node_index, Direction::Incoming)
    }
    pub fn children(&mut self, node_index: NodeIndex) -> Neighbors<()> {
        self.graph.neighbors_directed(node_index, Direction::Outgoing)
    }

    pub fn update_layout(&mut self, solver: &mut LimnSolver) {
        let mut dfs = Dfs::new(&self.graph, self.root_index.unwrap());
        while let Some(node_index) = dfs.next(&self.graph) {
            let ref mut widget = self.graph[node_index];
            widget.layout.update(solver);
        }
    }
    pub fn draw_node(&mut self,
                     context: Context,
                     graphics: &mut G2d,
                     node_index: NodeIndex,
                     crop_to: Rectangle) {

        let crop_to = {
            let ref mut widget = self.graph[node_index];
            widget.draw(crop_to, &mut self.glyph_cache, context, graphics);

            util::crop_rect(crop_to, widget.layout.bounds())
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
                let bounds = widget.layout.bounds();
                util::draw_rect_outline(bounds, color, context, graphics);
            }
        }
    }
    pub fn add_widget(&mut self,
                      widget: WidgetBuilder,
                      parent_index: Option<NodeIndex>,
                      solver: &mut LimnSolver)
                      -> NodeIndex {

        let (children, constraints, widget) = widget.build();
        solver.add_widget(&widget, constraints);

        let id = widget.id;
        let widget_index = self.graph.add_node(widget);
        if let Some(parent_index) = parent_index {
            self.graph.add_edge(parent_index, widget_index, ());
        }
        self.widget_map.insert(id, widget_index);
        self.dirty_widgets.insert(widget_index);
        for child in children {
            self.add_widget(child, Some(widget_index), solver);
        }
        widget_index
    }

    pub fn remove_widget(&mut self, widget_id: WidgetId, solver: &mut LimnSolver) {
        if let Some(node_index) = self.find_widget(widget_id) {
            self.graph.remove_node(node_index);
            self.dirty_widgets.insert(self.root_index.unwrap());
            solver.remove_widget(&widget_id);
        }
    }
    pub fn get_widget(&self, widget_id: WidgetId) -> Option<&mut Widget> {
        self.widget_map.get(&widget_id).and_then(|node_index| {
            let ref widget = self.graph[NodeIndex::new(node_index.index())];
            return Some(widget);
        });
        None
    }
    pub fn is_mouse_over(&mut self, node_index: NodeIndex, mouse: Point) -> bool {
        let ref mut widget = self.graph[node_index];
        widget.is_mouse_over(mouse)
    }
    pub fn find_widget(&mut self, widget_id: WidgetId) -> Option<NodeIndex> {
        self.widget_map.get(&widget_id).map(|index| *index)
    }
    pub fn get_widget_index(&mut self, index: NodeIndex) -> Option<&Widget> {
        self.graph.node_weight(index)
    }

    pub fn trigger_widget_event(&mut self,
                                node_index: NodeIndex,
                                type_id: TypeId,
                                data: &Box<Any + Send>,
                                event_queue: &mut EventQueue,
                                input_state: &InputState,
                                solver: &mut LimnSolver)
                                -> bool {
        let ref mut widget = self.graph[node_index];
        let handled = widget.trigger_event(type_id,
                                           data,
                                           event_queue,
                                           solver,
                                           input_state);
        if let Some(ref mut drawable) = widget.drawable {
            if drawable.has_updated {
                self.dirty_widgets.insert(node_index);
                drawable.has_updated = false;
            }
        }
        handled
    }

    pub fn handle_subtree_event(&mut self, id: WidgetId, type_id: TypeId, data: &Box<Any + Send>, event_queue: &mut EventQueue, input_state: &InputState,
                                solver: &mut LimnSolver) {
        if let Some(node_index) = self.find_widget(id) {
            let mut dfs = Dfs::new(&self.graph, node_index);
            while let Some(node_index) = dfs.next(&self.graph) {
                self.trigger_widget_event(node_index, type_id, data, event_queue, input_state, solver);
            }
        }
    }

    pub fn handle_undermouse_event(&mut self, type_id: TypeId, data: &Box<Any + Send>, event_queue: &mut EventQueue, input_state: &mut InputState,
                                solver: &mut LimnSolver) {
        let mut dfs = DfsPostOrder::new(&self.graph, self.root_index.unwrap());
        while let Some(node_index) = dfs.next(&self.graph) {
            let is_mouse_over = self.is_mouse_over(node_index, input_state.mouse);
            if is_mouse_over {
                let handled = self.trigger_widget_event(node_index, type_id, data, event_queue, input_state, solver);
                let ref mut widget = self.graph[node_index];
                input_state.last_over.insert(widget.id);
                // for now just one widget can handle an event, later, just don't send to parents
                // not no other widgets
                if handled {
                    return;
                }
            }
        }
    }
}
