use std::collections::HashMap;
use std::any::{Any, TypeId};

use petgraph::stable_graph::StableGraph;
use petgraph::graph::NodeIndex;
use petgraph::visit::{Dfs, DfsPostOrder};
use petgraph::Direction;
use petgraph::visit::Visitable;
use petgraph::stable_graph::WalkNeighbors;

use cassowary::strength::*;

use graphics;
use graphics::Context;

use backend::gfx::G2d;
use backend::glyph::GlyphCache;
use backend::window::Window;

use widget::{Widget, WidgetContainer};
use widget::WidgetBuilder;
use widget::WidgetBuilderCore;
use widget::layout::LayoutVars;
use util::{self, Point, Rectangle, Dimensions};
use resources::{resources, WidgetId};
use color::*;
use event::Target;

use ui::solver::LimnSolver;
use event::Queue;

const DEBUG_BOUNDS: bool = false;

type Graph = StableGraph<WidgetContainer, ()>;

pub struct WidgetGraph {
    pub graph: GraphWrapper,
    queue: Queue,
    redraw: u32,
    glyph_cache: GlyphCache,
}

impl WidgetGraph {
    pub fn new(window: &mut Window, queue: Queue) -> Self {
        WidgetGraph {
            graph: GraphWrapper::new(),
            queue: queue,
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
        self.graph.root_id = root_widget.id();
        root_widget.layout().top_left(Point { x: 0.0, y: 0.0 });
        self.add_widget(root_widget, None, solver);
        let ref mut root = self.graph.get_root();
        solver.update_solver(|solver| {
            solver.add_edit_variable(root.layout.right, STRONG).unwrap();
            solver.add_edit_variable(root.layout.bottom, STRONG).unwrap();
        });
    }
    pub fn get_root_dims(&mut self) -> Dimensions {
        //let root_index = self.root_index();
        //let ref mut root = &mut self.graph.graph[root_index].widget;
        let root = self.graph.get_root();
        let mut dims = root.layout.get_dims();
        // use min size to prevent window size from being set to 0 (X crashes)
        dims.width = f64::max(100.0, dims.width);
        dims.height = f64::max(100.0, dims.height);
        dims
    }
    pub fn window_resized(&mut self, window_dims: Dimensions, solver: &mut LimnSolver) {
        //let root_index = self.root_index();
        //let ref mut root = self.graph.graph[root_index].widget;
        let root = self.graph.get_root();
        solver.update_solver(|solver| {
            solver.suggest_value(root.layout.right, window_dims.width).unwrap();
            solver.suggest_value(root.layout.bottom, window_dims.height).unwrap();
        });
        self.redraw = 2;
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
        let crop_to = Rectangle::new_from_pos_dim(Point::zero(), Dimensions::max());
        let id = self.graph.root_id;
        self.draw_node(context, graphics, id, crop_to);
        if DEBUG_BOUNDS {
            let root_id = self.graph.root_id;
            let mut dfs = self.graph.dfs(root_id);
            //Dfs::new(&self.graph.graph, self.root_index());
            while let Some(widget_id) = dfs.next(&self.graph.graph) {
                let widget = self.graph.get_widget(widget_id).unwrap();
                let color = widget.debug_color.unwrap_or(GREEN);
                let bounds = widget.layout.bounds();
                util::draw_rect_outline(bounds, color, context, graphics);
            }
        }
    }
    pub fn draw_node(&mut self,
                     context: Context,
                     graphics: &mut G2d,
                     widget_id: WidgetId,
                     crop_to: Rectangle) {

        let crop_to = {
            let ref mut widget = self.graph.get_widget(widget_id).unwrap();
            widget.draw(crop_to, &mut self.glyph_cache, context, graphics);
            util::crop_rect(crop_to, widget.layout.bounds())
        };

        if !crop_to.no_area() {
            let children: Vec<WidgetId> = self.graph.children(widget_id).collect(&self.graph.graph);
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
            if let Some(parent) = self.graph.get_widget(parent_id) {
                if parent.bound_children {
                    widget.layout().bound_by(&parent.layout);
                }
            }
        }
        let (children, constraints, widget) = widget.build();
        solver.add_widget(&widget.widget, constraints);

        let id = widget.widget.id;
        let layout = widget.widget.layout.clone();
        let widget_index = self.graph.add_widget(widget, parent_id);
        if let Some(parent_id) = parent_id {
            self.queue.push(Target::Widget(parent_id), ChildAttachedEvent(id, layout));
        }
        self.redraw();
        for child in children {
            self.add_widget(child, Some(id), solver);
        }
        widget_index
    }

    pub fn remove_widget(&mut self, widget_id: WidgetId, solver: &mut LimnSolver) {
        if let Some(widget) = self.graph.remove_widget(widget_id) {
            self.redraw();
            solver.remove_widget(&widget.widget.layout);
        }
    }

    pub fn widget_under_cursor(&mut self, point: Point) -> Option<WidgetId> {
        // first widget found is the deepest, later will need to have z order as ordering
        self.graph.widgets_under_cursor(point).next(&mut self.graph.graph)
    }

    fn handle_widget_event(&mut self,
                           widget_id: WidgetId,
                           type_id: TypeId,
                           data: &Box<Any + Send>,
                           queue: &mut Queue,
                           solver: &mut LimnSolver) -> bool
    {
        if let Some(widget_container) = self.graph.get_widget_container(widget_id) {
            let handled = widget_container.trigger_event(type_id,
                                                     data,
                                                     queue,
                                                     solver);
            if widget_container.widget.has_updated {
                self.redraw = 2;
                widget_container.widget.has_updated = false;
            }
            handled
        } else {
            false
        }
    }

    pub fn handle_event(&mut self,
                        address: Target,
                        type_id: TypeId,
                        data: &Box<Any + Send>,
                        queue: &mut Queue,
                        solver: &mut LimnSolver) {
        match address {
            Target::Widget(id) => {
                self.handle_widget_event(id, type_id, data, queue, solver);
            }
            Target::Child(id) => {
                if let Some(child_id) = self.graph.children(id).next(&self.graph.graph) {
                    self.handle_widget_event(child_id, type_id, data, queue, solver);
                }
            }
            Target::SubTree(id) => {
                let mut dfs = self.graph.dfs(id);
                while let Some(widget_id) = dfs.next(&self.graph.graph) {
                    self.handle_widget_event(widget_id, type_id, data, queue, solver);
                }
            }
            Target::BubbleUp(id) => {
                // bubble up event from widget, until either it reaches the root, or some widget handles it
                let mut maybe_id = Some(id);
                while let Some(id) = maybe_id {
                    let handled = self.handle_widget_event(id, type_id, data, queue, solver);
                    maybe_id = if handled { None } else { self.graph.parent(id) };
                }
            }
            _ => ()
        }
    }
}
pub struct ChildAttachedEvent(pub WidgetId, pub LayoutVars);

pub struct GraphWrapper {
    pub graph: Graph,
    pub root_id: WidgetId,
    widget_map: HashMap<WidgetId, NodeIndex>,
}
impl GraphWrapper {
    fn new() -> Self {
        GraphWrapper {
            graph: StableGraph::new(),
            widget_map: HashMap::new(),
            root_id: resources().widget_id(),
        }
    }

    pub fn find_widget(&self, widget_id: WidgetId) -> Option<NodeIndex> {
        self.widget_map.get(&widget_id).map(|index| *index)
    }
    pub fn get_widget(&mut self, widget_id: WidgetId) -> Option<&mut Widget> {
        if let Some(node_index) = self.widget_map.get(&widget_id) {
            if let Some(widget_container) = self.graph.node_weight_mut(node_index.clone()) {
                return Some(&mut widget_container.widget)
            }
        }
        None
    }
    pub fn get_widget_container(&mut self, widget_id: WidgetId) -> Option<&mut WidgetContainer> {
        if let Some(node_index) = self.widget_map.get(&widget_id) {
            if let Some(widget_container) = self.graph.node_weight_mut(node_index.clone()) {
                return Some(widget_container)
            }
        }
        None
    }

    pub fn add_widget(&mut self,
                      widget: WidgetContainer,
                      parent_id: Option<WidgetId>)
                      -> NodeIndex
    {
        let id = widget.widget.id;
        let widget_index = self.graph.add_node(widget);
        self.widget_map.insert(id, widget_index);
        if let Some(parent_id) = parent_id {
            if let Some(parent_index) = self.find_widget(parent_id) {
                self.graph.add_edge(parent_index, widget_index, ());
            }
        }
        widget_index
    }

    pub fn remove_widget(&mut self, widget_id: WidgetId) -> Option<WidgetContainer> {
        if let Some(node_index) = self.find_widget(widget_id) {
            self.widget_map.remove(&widget_id);
            if let Some(widget) = self.graph.remove_node(node_index) {
                return Some(widget);
            }
        }
        None
    }
    fn root_index(&self) -> NodeIndex {
        self.find_widget(self.root_id).unwrap()
    }
    pub fn get_root(&mut self) -> &mut Widget {
        let root_id = self.root_id;
        self.get_widget(root_id).unwrap()
    }

    fn parent(&mut self, widget_id: WidgetId) -> Option<WidgetId> {
        let node_index = if let Some(node_index) = self.widget_map.get(&widget_id) {
            node_index.clone()
        } else {
            NodeIndex::end()
        };
        NeighborsIter::new(&self.graph, node_index, Direction::Incoming).next(&self.graph)
    }
    fn children(&mut self, widget_id: WidgetId) -> NeighborsIter {
        let node_index = if let Some(node_index) = self.widget_map.get(&widget_id) {
            node_index.clone()
        } else {
            NodeIndex::end()
        };
        NeighborsIter::new(&self.graph, node_index, Direction::Outgoing)
    }
    pub fn widgets_under_cursor(&mut self, point: Point) -> CursorWidgetIter {
        CursorWidgetIter::new(point, &self.graph, self.root_index())
    }
    pub fn dfs(&mut self, widget_id: WidgetId) -> DfsIter {
        let node_index = self.widget_map.get(&widget_id).unwrap();
        DfsIter::new(&self.graph, node_index.clone())
    }
}

struct NeighborsIter {
    neighbors: WalkNeighbors<u32>,
}
impl NeighborsIter {
    fn new(graph: &Graph, node_index: NodeIndex, direction: Direction) -> Self {
        NeighborsIter {
            neighbors: graph.neighbors_directed(node_index, direction).detach()
        }
    }
    fn next(&mut self, graph: &Graph) -> Option<WidgetId> {
        if let Some((_, node_index)) = self.neighbors.next(graph) {
            Some(graph[node_index].widget.id)
        } else {
            None
        }
    }
    fn collect(&mut self, graph: &Graph) -> Vec<WidgetId> {
        let mut ids = Vec::new();
        while let Some(id) = self.next(graph) {
            ids.push(id);
        }
        ids
    }
}

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
pub struct DfsIter {
    dfs: Dfs<NodeIndex, <Graph as Visitable>::Map>,
}
impl DfsIter {
    pub fn new(graph: &Graph, root_index: NodeIndex) -> Self {
        DfsIter {
            dfs: Dfs::new(graph, root_index),
        }
    }
    pub fn next(&mut self, graph: &Graph) -> Option<WidgetId> {
        if let Some(node_index) = self.dfs.next(graph) {
            Some(graph[node_index].widget.id)
        } else {
            None
        }
    }
}