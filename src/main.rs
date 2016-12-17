extern crate backend;
extern crate graphics;
extern crate cassowary;
extern crate input;
extern crate window;
extern crate petgraph;

use input::{ResizeEvent, MouseCursorEvent};
use backend::{Window, WindowEvents, OpenGL};
use backend::gfx::G2d;
use graphics::*;
use graphics::types::{Color, Scalar};
use window::Size;

use cassowary::{ Solver, Variable, Constraint };
use cassowary::WeightedRelation::*;
use cassowary::strength::{ WEAK, MEDIUM, STRONG, REQUIRED };

use petgraph::Graph;
use petgraph::graph::NodeIndex;
use petgraph::visit::{ Dfs, DfsPostOrder, Walker };

pub mod widget;
use widget::*;

#[derive(Copy, Clone, Debug)]
pub struct Dimensions { width: Scalar, height: Scalar, }
#[derive(Copy, Clone, Debug)]
pub struct Point { x: Scalar, y: Scalar, }

impl Into<Size> for Dimensions {
    fn into(self) -> Size {
        Size { width: self.width as u32, height: self.height as u32 }
    }
}
impl Into<Point> for [f64; 2] {
    fn into(self) -> Point {
        Point { x: self[0], y: self[1] }
    }
}

#[derive(Copy, Clone)]
pub struct Event {}
pub trait EventListener {
    fn handle_event(&self, event: Event);
    fn matches(&self, event: Event) -> bool {
        true
    }
}
struct EventBus<'a> {
    listeners: Vec<&'a EventListener>,
}
impl<'a> EventBus<'a> {
    fn find_listeners(&self, event: Event) -> Vec<&'a EventListener> {
        let mut matching_listeners = Vec::new();
        let ref listeners = self.listeners;
        for listener in listeners {
            if listener.matches(event) {
                matching_listeners.push(*listener);
            }
        }
        matching_listeners
    }
    fn publish(&self, event: Event) {
        let listeners = self.find_listeners(event);
        for listener in listeners {
            listener.handle_event(event);
        }
    }
}
struct Ui<'a> {
    graph: Graph<Widget<'a>, ()>,
    window: NodeIndex,
    constraints: Vec<Constraint>,
    pub solver: Solver,
    window_width: Variable,
    window_height: Variable,
    event_bus: EventBus<'a>,
}
impl<'a> Ui<'a> {
    fn new(window_dim: Dimensions) -> Self {
        let window = Widget::new(None);
        let window_width = Variable::new();
        let window_height = Variable::new();
        let mut constraints = Vec::new();
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
            event_bus: EventBus { listeners: Vec::new() },
        }
    }
    fn resize_window(&mut self, window_dims: [u32; 2]) {
        self.solver.suggest_value(self.window_width, window_dims[0] as f64).unwrap();
        self.solver.suggest_value(self.window_height, window_dims[1] as f64).unwrap();
    }
    fn init(&mut self) {
        let mut dfs = Dfs::new(&self.graph, self.window);
        while let Some(node_index) = dfs.next(&self.graph) {

            let ref mut node = self.graph[node_index];
            let constraints = &mut node.layout.constraints;
            self.constraints.append(constraints);
        }
        self.solver.add_constraints(&self.constraints);
    }
    fn draw(&mut self, c: Context, g: &mut G2d) {
        let mut dfs = Dfs::new(&self.graph, self.window);
        while let Some(node_index) = dfs.next(&self.graph) {
            let ref node = self.graph[node_index];
            node.draw(&mut self.solver, c, g);
        }
    }
    fn add_widget(&mut self, parent_index: NodeIndex, child: Widget<'a>) -> NodeIndex {
        let child_index = self.graph.add_node(child);
        self.graph.add_edge(parent_index, child_index, ());

        let (parent, child) = self.graph.index_twice_mut(parent_index, child_index);
        child.layout.bound_by(&parent.layout);
        
        self.event_bus.listeners.append(&mut child.listeners);

        child_index
    }
    fn check_mouse_over(&mut self, pos: Point) {
        let mut dfs = DfsPostOrder::new(&self.graph, self.window);
        while let Some(node_index) = dfs.next(&self.graph) {
            let ref node = self.graph[node_index];
            if node.is_mouse_over(&mut self.solver, pos) {
                println!("found widget");
            }
        }
    }
}
fn main() {
    let window_dim = Dimensions { width: 400.0, height: 720.0 };

    // Construct the window.
    let mut window: Window =
        backend::window::WindowSettings::new("Grafiki Demo", window_dim)
            .opengl(OpenGL::V3_2).samples(4).exit_on_esc(true).build().unwrap();

    // Create the event loop.
    let mut events = WindowEvents::new();

    let circle2 = EllipseDrawable::new([1.0, 1.0, 1.0]);
    let mut box3 = Widget::new(Some(&circle2));
    let rect = RectDrawable::new([1.0, 0.0, 0.0]);
    let mut box1 = Widget::new(Some(&rect));
    let circle = EllipseDrawable::new([1.0, 0.0, 1.0]);

    struct ClickListener {
    }
    impl EventListener for ClickListener {
        fn handle_event(&self, event: Event) {
            println!("event");
        }
    }
    let listener = ClickListener {};
    let mut box2 = Widget::new(Some(&circle));
    box2.listeners.push(&listener);

    let ui = &mut Ui::new(window_dim);

    let box1_constraints = [
        box1.layout.top |EQ(REQUIRED)| 0.0,
        box1.layout.left |EQ(REQUIRED)| 0.0,
        box1.layout.left |LE(REQUIRED)| box1.layout.right];
    box1.layout.width(50.0, WEAK);
    box1.layout.height(100.0, WEAK);
    box1.layout.add_constraints(&box1_constraints);

    let box2_constraints = [
        box2.layout.bottom |EQ(REQUIRED)| ui.window_height, // bottom align
        box2.layout.right |EQ(REQUIRED)| ui.window_width, // right align
        box2.layout.left |GE(REQUIRED)| box1.layout.right, // no overlap
        box2.layout.left |LE(REQUIRED)| box2.layout.right];
    box2.layout.width(100.0, WEAK);
    box2.layout.height(100.0, WEAK);
    box2.layout.add_constraints(&box2_constraints);

    let window_index = ui.window;
    let box1_index = ui.add_widget(window_index, box1);
    ui.add_widget(window_index, box2);
    ui.add_widget(box1_index, box3);
    ui.init();

    // Poll events from the window.
    while let Some(event) = events.next(&mut window) {
        window.handle_event(&event);
        if let Some(window_dims) = event.resize_args() {
            ui.resize_window(window_dims);
        }
        if let Some(xy) = event.mouse_cursor_args() {
            ui.check_mouse_over(xy.into());
            ui.event_bus.publish(Event {});
        }

        window.draw_2d(&event, |c, g| {
            clear([0.8, 0.8, 0.8, 1.0], g);
            ui.draw(c, g);
        });
    }
}