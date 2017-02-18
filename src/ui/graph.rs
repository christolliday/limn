use std::collections::{HashSet, HashMap};
use std::f64;
use std::any::{Any, TypeId};

use petgraph::stable_graph::StableGraph;
use petgraph::graph::NodeIndex;
use petgraph::visit::{Dfs, DfsPostOrder};
use petgraph::Direction;
use petgraph::stable_graph::Neighbors;

use cassowary::strength::*;

use graphics;
use graphics::Context;

use backend::gfx::G2d;
use backend::glyph::GlyphCache;
use backend::window::Window;

use widget::Widget;
use widget::builder::WidgetBuilder;
use util::{self, Point, Rectangle, Dimensions};
use resources::{resources, WidgetId};
use color::*;
use ui::EventAddress;

use super::layout::LimnSolver;
use super::queue::EventQueue;
use super::InputState;

const DEBUG_BOUNDS: bool = false;

pub struct WidgetGraph {
    graph: StableGraph<Widget, ()>,
    pub root_id: WidgetId,
    widget_map: HashMap<WidgetId, NodeIndex>,
    dirty_widgets: HashSet<WidgetId>,
    glyph_cache: GlyphCache,
}

impl WidgetGraph {
    pub fn new(window: &mut Window) -> Self {
        WidgetGraph {
            graph: StableGraph::<Widget, ()>::new(),
            root_id: resources().widget_id(),
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
        self.root_id = root_widget.id;
        root_widget.layout.top_left(Point { x: 0.0, y: 0.0 }, None);
        self.add_widget(root_widget, None, solver);
        let ref mut root = self.get_root();
        solver.update_solver(|solver| {
            solver.add_edit_variable(root.layout.right, STRONG).unwrap();
            solver.add_edit_variable(root.layout.bottom, STRONG).unwrap();
        });
    }
    pub fn get_root(&mut self) -> &mut Widget {
        let root_id = self.root_id;
        self.get_widget(root_id).unwrap()
    }
    pub fn get_root_dims(&mut self, solver: &mut LimnSolver) -> Dimensions {
        let root_index = self.root_index();
        let ref mut root = &mut self.graph[root_index];
        root.layout.update(solver);
        root.layout.get_dims()
    }
    fn root_index(&self) -> NodeIndex {
        self.widget_map[&self.root_id].clone()
    }
    pub fn window_resized(&mut self, window_dims: Dimensions, solver: &mut LimnSolver) {
        let root_index = self.root_index();
        let ref mut root = self.graph[root_index];
        root.layout.update(solver);
        solver.update_solver(|solver| {
            solver.suggest_value(root.layout.right, window_dims.width).unwrap();
            solver.suggest_value(root.layout.bottom, window_dims.height).unwrap();
        });
        self.dirty_widgets.insert(self.root_id);
    }
    pub fn parents(&mut self, node_index: NodeIndex) -> Neighbors<()> {
        self.graph.neighbors_directed(node_index, Direction::Incoming)
    }
    pub fn children(&mut self, node_index: NodeIndex) -> Neighbors<()> {
        self.graph.neighbors_directed(node_index, Direction::Outgoing)
    }

    pub fn update_layout(&mut self, solver: &mut LimnSolver) {
        let mut dfs = Dfs::new(&self.graph, self.root_index());
        while let Some(node_index) = dfs.next(&self.graph) {
            let ref mut widget = self.graph[node_index];
            widget.layout.update(solver);
        }
    }
    pub fn redraw(&mut self) {
        self.dirty_widgets.insert(self.root_id);
    }
    pub fn draw_if_needed(&mut self, window: &mut Window) {
        if self.dirty_widgets.len() > 0 {
            window.draw_2d(|context, graphics| {
                graphics::clear([0.8, 0.8, 0.8, 1.0], graphics);
                self.draw(context, graphics);
            });
            self.dirty_widgets.clear();
        }
    }
    pub fn draw(&mut self, context: Context, graphics: &mut G2d) {
        let index = self.root_index().clone();
        let crop_to = Rectangle::new_from_pos_dim(Point::zero(), Dimensions::max());
        self.draw_node(context, graphics, index, crop_to);
        if DEBUG_BOUNDS {
            let mut dfs = Dfs::new(&self.graph, self.root_index());
            while let Some(node_index) = dfs.next(&self.graph) {
                let ref widget = self.graph[node_index];
                let color = widget.debug_color.unwrap_or(GREEN);
                let bounds = widget.layout.bounds();
                util::draw_rect_outline(bounds, color, context, graphics);
            }
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
        self.dirty_widgets.insert(id);
        for child in children {
            self.add_widget(child, Some(widget_index), solver);
        }
        widget_index
    }

    pub fn remove_widget(&mut self, widget_id: WidgetId, solver: &mut LimnSolver) {
        if let Some(node_index) = self.find_widget(widget_id) {
            self.graph.remove_node(node_index);
            self.dirty_widgets.insert(self.root_id);
            solver.remove_widget(&widget_id);
        }
    }
    pub fn get_widget(&mut self, widget_id: WidgetId) -> Option<&mut Widget> {
        if let Some(node_index) = self.widget_map.get(&widget_id) {
            self.graph.node_weight_mut(node_index.clone())
        } else {
            None
        }
    }
    pub fn is_mouse_over(&mut self, node_index: NodeIndex, mouse: Point) -> bool {
        let ref mut widget = self.graph[node_index];
        widget.is_mouse_over(mouse)
    }
    pub fn find_widget(&mut self, widget_id: WidgetId) -> Option<NodeIndex> {
        self.widget_map.get(&widget_id).map(|index| *index)
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
                self.dirty_widgets.insert(widget.id);
                drawable.has_updated = false;
            }
        }
        handled
    }

    pub fn handle_event(&mut self,
                        address: EventAddress,
                        type_id: TypeId,
                        data: &Box<Any + Send>,
                        event_queue: &mut EventQueue,
                        input_state: &mut InputState,
                        solver: &mut LimnSolver) {
        match address {
            EventAddress::Widget(id) => {
                if let Some(node_index) = self.find_widget(id) {
                    self.trigger_widget_event(node_index, type_id, data, event_queue, input_state, solver);
                }
            }
            EventAddress::Child(id) => {
                if let Some(node_index) = self.find_widget(id) {
                    if let Some(child_index) = self.children(node_index).next() {
                        self.trigger_widget_event(child_index, type_id, data, event_queue, input_state, solver);
                    }
                }
            }
            EventAddress::SubTree(id) => {
                if let Some(node_index) = self.find_widget(id) {
                    let mut dfs = Dfs::new(&self.graph, node_index);
                    while let Some(node_index) = dfs.next(&self.graph) {
                        self.trigger_widget_event(node_index, type_id, data, event_queue, input_state, solver);
                    }
                }
            }
            EventAddress::UnderMouse => {
                let mut dfs = DfsPostOrder::new(&self.graph, self.root_index());
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
            _ => ()
        }
    }
}
