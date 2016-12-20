
use backend::gfx::G2d;

use petgraph::Graph;
use petgraph::graph::NodeIndex;
use petgraph::visit::{ Dfs };

use input::{Event, Input, Motion};

use cassowary::{ Solver, Variable, Constraint };
use cassowary::strength::*;

use graphics::Context;
use super::widget::*;
use super::util::*;

pub struct Ui {
    graph: Graph<Widget, ()>,
    pub window: NodeIndex,
    constraints: Vec<Constraint>,
    pub solver: Solver,
    pub window_width: Variable,
    pub window_height: Variable,
}
impl Ui {
    pub fn new(window_dim: Dimensions) -> Self {
        let window = Widget::new(None);
        let window_width = Variable::new();
        let window_height = Variable::new();
        let constraints = Vec::new();
        let mut solver = Solver::new();
        solver.add_edit_variable(window_width, STRONG).unwrap();
        solver.add_edit_variable(window_height, STRONG).unwrap();
        solver.suggest_value(window_width, window_dim.width).unwrap();
        solver.suggest_value(window_height, window_dim.height).unwrap();

        let mut graph = Graph::<Widget, ()>::new();
        let window = graph.add_node(window);
        Ui {
            graph: graph, window: window,
            solver: solver, constraints: constraints,
            window_width: window_width, window_height: window_height,
        }
    }
    pub fn resize_window(&mut self, window_dims: [u32; 2]) {
        self.solver.suggest_value(self.window_width, window_dims[0] as f64).unwrap();
        self.solver.suggest_value(self.window_height, window_dims[1] as f64).unwrap();
    }
    pub fn init(&mut self) {
        let mut dfs = Dfs::new(&self.graph, self.window);
        while let Some(node_index) = dfs.next(&self.graph) {

            let ref mut node = self.graph[node_index];
            let constraints = &mut node.layout.constraints;
            self.constraints.append(constraints);
        }
        self.solver.add_constraints(&self.constraints).unwrap();
    }
    pub fn draw(&mut self, c: Context, g: &mut G2d) {
        let mut dfs = Dfs::new(&self.graph, self.window);
        while let Some(node_index) = dfs.next(&self.graph) {
            let ref widget = self.graph[node_index];
            widget.draw(&mut self.solver, c, g);
        }
    }
    pub fn add_widget(&mut self, parent_index: NodeIndex, child: Widget) -> NodeIndex {
        let child_index = self.graph.add_node(child);
        self.graph.add_edge(parent_index, child_index, ());

        let (parent, child) = self.graph.index_twice_mut(parent_index, child_index);
        child.layout.bound_by(&parent.layout);

        child_index
    }
    pub fn post_event(&mut self, event: &Event) {
        let mut dfs = Dfs::new(&self.graph, self.window);
        while let Some(node_index) = dfs.next(&self.graph) {
            let ref widget = self.graph[node_index];
            match event {
                &Event::Input(Input::Move(Motion::MouseCursor(x, y))) => {
                    let pos = Point{x: x, y: y};
                    for listener in &widget.listeners {
                        if widget.is_mouse_over(&mut self.solver, pos) && listener.matches(event) {
                            listener.handle_event(event);
                        }
                    }
                }, _ => {}
            }
        }
    }
}