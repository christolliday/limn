extern crate backend;
extern crate graphics;
extern crate cassowary;
extern crate input;
extern crate window;

use input::ResizeEvent;
use backend::{Window, WindowEvents, OpenGL};
use backend::gfx::G2d;
use graphics::*;
use graphics::types::{Color, Scalar};
use window::Size;

use cassowary::{ Solver, Variable, Constraint };
use cassowary::WeightedRelation::*;
use cassowary::strength::{ WEAK, MEDIUM, STRONG, REQUIRED };

#[derive(Copy, Clone)]
struct Dimensions { width: Scalar, height: Scalar, }
struct Point { x: Scalar, y: Scalar, }

impl Into<Size> for Dimensions {
    fn into(self) -> Size {
        Size { width: self.width as u32, height: self.height as u32 }
    }
}

trait WidgetDrawable {
    fn draw(&self, bounds: types::Rectangle, c: Context, g: &mut G2d);
}

struct RectDrawable {
    background: Color,
}
impl RectDrawable {
    fn new(color: [f32; 3]) -> Self {
        RectDrawable { background: [color[0], color[1], color[2], 1.0] }
    }
}
impl WidgetDrawable for RectDrawable {
    fn draw(&self, bounds: types::Rectangle, c: Context, g: &mut G2d) {
        Rectangle::new(self.background).draw(bounds, &c.draw_state, c.transform, g);
    }
}

struct EllipseDrawable {
    background: Color,
}
impl EllipseDrawable {
    fn new(color: [f32; 3]) -> Self {
        EllipseDrawable { background: [color[0], color[1], color[2], 1.0] }
    }
}
impl WidgetDrawable for EllipseDrawable {
    fn draw(&self, bounds: types::Rectangle, c: Context, g: &mut G2d) {
        Ellipse::new(self.background).draw(bounds, &c.draw_state, c.transform, g);
    }
}

struct Node<'a> {
    left: Variable,
    right: Variable,
    top: Variable,
    bottom: Variable,
    drawable: Option<&'a WidgetDrawable>,
    constraints: Vec<Constraint>,
    nodes: Vec<Node<'a>>,
}

impl<'a>  Node<'a>  {
    fn new(drawable: Option<&'a WidgetDrawable>) -> Self {
        Node {
            left: Variable::new(),
            right: Variable::new(),
            top: Variable::new(),
            bottom: Variable::new(),
            drawable: drawable,
            constraints: Vec::new(),
            nodes: Vec::new(),
        }
    }
    fn print(&self, solver: &mut Solver) {
        println!("{:?} {:?} {:?} {:?}",
            solver.get_value(self.left),
            solver.get_value(self.top),
            solver.get_value(self.right),
            solver.get_value(self.bottom));
    }
    fn draw(&self, solver: &mut Solver, c: Context, g: &mut G2d) {
        if let Some(drawable) = self.drawable {
            let bounds = [
                    solver.get_value(self.left),
                    solver.get_value(self.top),
                    solver.get_value(self.right) - solver.get_value(self.left),
                    solver.get_value(self.bottom) - solver.get_value(self.top),
                    ];
            drawable.draw(bounds, c, g);
        }
    }
    // layout
    fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }
    fn add_constraints(&mut self, constraints: &[Constraint]) {
        self.constraints.extend_from_slice(constraints);
    }
    fn width(&mut self, width: Scalar, strength: f64) {
        self.constraints.push(self.right - self.left |EQ(strength)| width)
    }
    fn height(&mut self, height: Scalar, strength: f64) {
        self.constraints.push(self.bottom - self.top |EQ(strength)| height)
    }
    fn add_node(&mut self, node: Node<'a>) {
        self.nodes.push(node);
    }
}

struct Ui<'a> {
    window: Node<'a>,
    constraints: Vec<Constraint>,
    pub solver: Solver,
    window_width: Variable,
    window_height: Variable,
}
impl<'a> Ui<'a> {
    fn new(window_dim: Dimensions) -> Self {
        let window = Node::new(None);
        let window_width = Variable::new();
        let window_height = Variable::new();
        let mut constraints = Vec::new();
        let mut solver = Solver::new();
        solver.add_edit_variable(window_width, STRONG).unwrap();
        solver.add_edit_variable(window_height, STRONG).unwrap();
        solver.suggest_value(window_width, window_dim.width).unwrap();
        solver.suggest_value(window_height, window_dim.height).unwrap();
        Ui { window: window, solver: solver, constraints: constraints, window_width: window_width, window_height: window_height }
    }
    fn resize_window(&mut self, window_dims: [u32; 2]) {
        self.solver.suggest_value(self.window_width, window_dims[0] as f64).unwrap();
        self.solver.suggest_value(self.window_height, window_dims[1] as f64).unwrap();
    }
    fn init(&mut self) {
        for ref mut node in &mut self.window.nodes {
            let constraints = &mut node.constraints;
            self.constraints.append(constraints);
        }
        self.solver.add_constraints(&self.constraints);
    }
    fn draw(&mut self, c: Context, g: &mut G2d) {
        for ref node in &self.window.nodes {
            node.draw(&mut self.solver, c, g);
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

    let rect = RectDrawable::new([1.0, 0.0, 0.0]);
    let mut box1 = Node::new(Some(&rect));
    let circle = EllipseDrawable::new([1.0, 0.0, 1.0]);
    let mut box2 = Node::new(Some(&circle));
    //let circle2 = EllipseDrawable::new([1.0, 1.0, 1.0]);
    //let mut box3 = Node::new(Some(&circle2));

    let ui = &mut Ui::new(window_dim);

    let box1_constraints = [
        box1.top |EQ(REQUIRED)| 0.0,
        box1.left |EQ(REQUIRED)| 0.0,
        box1.left |LE(REQUIRED)| box1.right];
    box1.width(50.0, WEAK);
    box1.height(100.0, WEAK);
    box1.add_constraints(&box1_constraints);

    let box2_constraints = [
        box2.bottom |EQ(REQUIRED)| ui.window_height, // bottom align
        box2.right |EQ(REQUIRED)| ui.window_width, // right align
        box2.left |GE(REQUIRED)| box1.right, // no overlap
        box2.left |LE(REQUIRED)| box2.right];
    box2.width(100.0, WEAK);
    box2.height(100.0, WEAK);
    box2.add_constraints(&box2_constraints);

    ui.window.add_node(box1);
    ui.window.add_node(box2);
    //box1.add_node(box3);
    ui.init();

    // Poll events from the window.
    while let Some(event) = events.next(&mut window) {
        window.handle_event(&event);
        if let Some(window_dims) = event.resize_args() {
            ui.resize_window(window_dims);
        }

        window.draw_2d(&event, |c, g| {
            clear([0.8, 0.8, 0.8, 1.0], g);
            ui.draw(c, g);
        });
    }
}