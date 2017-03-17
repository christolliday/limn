use std::collections::HashMap;
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

use widget::{Widget, WidgetContainer};
use widget::WidgetBuilder;
use util::{self, Point, Rectangle, Dimensions};
use resources::{resources, WidgetId};
use color::*;
use event::Target;

use ui::solver::LimnSolver;
use event::Queue;

const DEBUG_BOUNDS: bool = false;

type Graph = StableGraph<WidgetContainer, ()>;

pub struct WidgetGraph {
    pub graph: Graph,
    pub root_id: WidgetId,
    widget_map: HashMap<WidgetId, NodeIndex>,
    redraw: u32,
    glyph_cache: GlyphCache,
}

impl WidgetGraph {
    pub fn new(window: &mut Window) -> Self {
        WidgetGraph {
            graph: StableGraph::new(),
            root_id: resources().widget_id(),
            widget_map: HashMap::new(),
            redraw: 2,
            glyph_cache: GlyphCache::new(&mut window.context.factory, 512, 512),
        }
    }
    pub fn resize_window_to_fit(&mut self, window: &Window) {
        let window_dims = self.get_root_dims();
        window.window.set_inner_size(window_dims.width as u32, window_dims.height as u32);
    }
    pub fn set_root(&mut self, mut root_widget: WidgetBuilder, solver: &mut LimnSolver) {
        root_widget.set_debug_name("root");
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
    pub fn get_root_dims(&mut self) -> Dimensions {
        let root_index = self.root_index();
        let ref mut root = &mut self.graph[root_index].widget;
        let mut dims = root.layout.get_dims();
        // use min size to prevent window size from being set to 0 (X crashes)
        dims.width = f64::max(100.0, dims.width);
        dims.height = f64::max(100.0, dims.height);
        dims
    }
    fn root_index(&self) -> NodeIndex {
        self.widget_map[&self.root_id].clone()
    }
    pub fn window_resized(&mut self, window_dims: Dimensions, solver: &mut LimnSolver) {
        let root_index = self.root_index();
        let ref mut root = self.graph[root_index].widget;
        solver.update_solver(|solver| {
            solver.suggest_value(root.layout.right, window_dims.width).unwrap();
            solver.suggest_value(root.layout.bottom, window_dims.height).unwrap();
        });
        self.redraw = 2;
    }
    fn parent(&mut self, node_index: NodeIndex) -> Option<NodeIndex> {
        self.graph.neighbors_directed(node_index, Direction::Incoming).next()
    }
    fn children(&mut self, node_index: NodeIndex) -> Neighbors<()> {
        self.graph.neighbors_directed(node_index, Direction::Outgoing)
    }

    pub fn redraw(&mut self) {
        self.redraw = 2;
    }
    pub fn draw_if_needed(&mut self, window: &mut Window) {
        if self.redraw > 0 {
            window.draw_2d(|context, graphics| {
                graphics::clear([0.8, 0.8, 0.8, 1.0], graphics);
                self.draw(context, graphics);
            });
            self.redraw -= 1;
        }
    }
    pub fn draw(&mut self, context: Context, graphics: &mut G2d) {
        let index = self.root_index().clone();
        let crop_to = Rectangle::new_from_pos_dim(Point::zero(), Dimensions::max());
        self.draw_node(context, graphics, index, crop_to);
        if DEBUG_BOUNDS {
            let mut dfs = Dfs::new(&self.graph, self.root_index());
            while let Some(node_index) = dfs.next(&self.graph) {
                let ref widget = self.graph[node_index].widget;
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
            let ref mut widget = self.graph[node_index].widget;
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
                      mut widget: WidgetBuilder,
                      parent_id: Option<WidgetId>,
                      solver: &mut LimnSolver)
                      -> NodeIndex {

        if let Some(parent_id) = parent_id {
            if let Some(parent) = self.get_widget(parent_id) {
                if let Some(ref add_child_fn) = parent.add_child_fn {
                    add_child_fn(&mut widget, &parent);
                } else {
                    widget.layout.bound_by(&parent.layout);
                }
            }
        }
        let (children, constraints, widget) = widget.build();
        solver.add_widget(&widget.widget, constraints);

        let id = widget.widget.id;
        let widget_index = self.graph.add_node(widget);
        if let Some(parent_id) = parent_id {
            if let Some(parent_index) = self.find_widget(parent_id) {
                self.graph.add_edge(parent_index, widget_index, ());
            }
        }
        self.widget_map.insert(id, widget_index);
        self.redraw();
        for child in children {
            self.add_widget(child, Some(id), solver);
        }
        widget_index
    }

    pub fn remove_widget(&mut self, widget_id: WidgetId, solver: &mut LimnSolver) {
        if let Some(node_index) = self.find_widget(widget_id) {
            if let Some(widget) = self.graph.remove_node(node_index) {
                self.redraw();
                solver.remove_widget(&widget.widget.layout);
            }
        }
    }
    pub fn get_widget(&mut self, widget_id: WidgetId) -> Option<&mut Widget> {
        if let Some(node_index) = self.widget_map.get(&widget_id) {
            if let Some(widget_container) = self.graph.node_weight_mut(node_index.clone()) {
                return Some(&mut widget_container.widget)
            }
        }
        None
    }
    pub fn find_widget(&mut self, widget_id: WidgetId) -> Option<NodeIndex> {
        self.widget_map.get(&widget_id).map(|index| *index)
    }

    fn trigger_widget_event(&mut self,
                                node_index: NodeIndex,
                                type_id: TypeId,
                                data: &Box<Any + Send>,
                                queue: &mut Queue,
                                solver: &mut LimnSolver)
                                -> bool {
        let ref mut widget_container = self.graph[node_index];
        let handled = widget_container.trigger_event(type_id,
                                                     data,
                                                     queue,
                                                     solver);
        if widget_container.widget.has_updated {
            self.redraw = 2;
            widget_container.widget.has_updated = false;
        }
        handled
    }

    pub fn widgets_under_cursor(&mut self, point: Point) -> CursorWidgetIter {
        CursorWidgetIter::new(point, &self.graph, self.root_index())
    }

    pub fn widget_under_cursor(&mut self, point: Point) -> Option<WidgetId> {
        // first widget found is the deepest, later will need to have z order as ordering
        CursorWidgetIter::new(point, &self.graph, self.root_index()).next(&mut self.graph)
    }

    pub fn handle_event(&mut self,
                        address: Target,
                        type_id: TypeId,
                        data: &Box<Any + Send>,
                        queue: &mut Queue,
                        solver: &mut LimnSolver) {
        match address {
            Target::Widget(id) => {
                if let Some(node_index) = self.find_widget(id) {
                    self.trigger_widget_event(node_index, type_id, data, queue, solver);
                }
            }
            Target::Child(id) => {
                if let Some(node_index) = self.find_widget(id) {
                    if let Some(child_index) = self.children(node_index).next() {
                        self.trigger_widget_event(child_index, type_id, data, queue, solver);
                    }
                }
            }
            Target::SubTree(id) => {
                if let Some(node_index) = self.find_widget(id) {
                    let mut dfs = Dfs::new(&self.graph, node_index);
                    while let Some(node_index) = dfs.next(&self.graph) {
                        self.trigger_widget_event(node_index, type_id, data, queue, solver);
                    }
                }
            }
            Target::BubbleUp(id) => {
                // bubble up event from widget, until either it reaches the root, or some widget handles it
                let mut maybe_node_index = self.find_widget(id);
                while let Some(node_index) = maybe_node_index {
                    let handled = self.trigger_widget_event(node_index, type_id, data, queue, solver);
                    maybe_node_index = if handled { None } else { self.parent(node_index) };
                }
            }
            _ => ()
        }
    }
}
use petgraph::visit::Visitable;
pub struct CursorWidgetIter {
    point: Point,
    dfs: DfsPostOrder<NodeIndex, <Graph as Visitable>::Map>,
}
impl CursorWidgetIter {
    pub fn new(point: Point, graph: &Graph, root_index: NodeIndex) -> Self {
        CursorWidgetIter {
            point: point,
            dfs: DfsPostOrder::new(graph, root_index),
        }
    }
    pub fn next(&mut self, graph: &Graph) -> Option<WidgetId> {
        while let Some(node_index) = self.dfs.next(graph) {
            let ref widget = graph[node_index].widget;
            if widget.is_mouse_over(self.point) {
                return Some(widget.id);
            }
        }
        None
    }
}