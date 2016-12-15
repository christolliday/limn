extern crate backend;
extern crate graphics;
extern crate cassowary;
extern crate input;

use input::ResizeEvent;
use backend::{Window, WindowEvents, OpenGL};
use backend::gfx::G2d;
use graphics::*;
use graphics::types::Color;

use cassowary::{ Solver, Variable };
use cassowary::WeightedRelation::*;
use cassowary::strength::{ WEAK, MEDIUM, STRONG, REQUIRED };

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
    drawable: &'a WidgetDrawable,
}

impl<'a>  Node<'a>  {
    fn new<W: WidgetDrawable>(drawable: &'a W) -> Self {
        Node {
            left: Variable::new(),
            right: Variable::new(),
            top: Variable::new(),
            bottom: Variable::new(),
            drawable: drawable,
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
        let bounds = [
                solver.get_value(self.left),
                solver.get_value(self.top),
                solver.get_value(self.right) - solver.get_value(self.left),
                solver.get_value(self.bottom) - solver.get_value(self.top),
                ];
        self.drawable.draw(bounds, c, g);
    }
}

fn main() {
    const WIN_W: f64 = 400.0;
    const WIN_H: f64 = 720.0;

    // Construct the window.
    let mut window: Window =
        backend::window::WindowSettings::new("Grafiki Demo", [WIN_W as u32, WIN_H as u32])
            .opengl(OpenGL::V3_2).samples(4).exit_on_esc(true).build().unwrap();

    // Create the event loop.
    let mut events = WindowEvents::new();

    let window_width = Variable::new();
    let window_height = Variable::new();

    let rect = &RectDrawable::new([1.0, 0.0, 0.0]);
    let box1 = Node::new(rect);
    let circle = CircleDrawable::new([1.0, 0.0, 1.0]);
    let box2 = Node::new(&circle);

    let mut solver = Solver::new();
    solver.add_constraints(&[
                            window_width |GE(REQUIRED)| 0.0, // positive window width
                            window_height |GE(REQUIRED)| 0.0, // positive window width
                            box1.top |EQ(REQUIRED)| 0.0, // top align
                            box2.bottom |EQ(REQUIRED)| window_height, // bottom align
                            box1.left |EQ(REQUIRED)| 0.0, // left align
                            box2.right |EQ(REQUIRED)| window_width, // right align
                            box2.left |GE(REQUIRED)| box1.right, // no overlap
                            // positive widths
                            box1.left |LE(REQUIRED)| box1.right,
                            box2.left |LE(REQUIRED)| box2.right,
                            // preferred widths:
                            box1.right - box1.left |EQ(WEAK)| 50.0,
                            box2.right - box2.left |EQ(WEAK)| 100.0,
                            // heights
                            box1.bottom - box1.top |EQ(WEAK)| 100.0,
                            box2.bottom - box2.top |EQ(WEAK)| 100.0],
                            ).unwrap();
    
    solver.add_edit_variable(window_width, STRONG).unwrap();
    solver.add_edit_variable(window_height, STRONG).unwrap();
    solver.suggest_value(window_width, WIN_W).unwrap();
    solver.suggest_value(window_height, WIN_H).unwrap();

    // Poll events from the window.
    while let Some(event) = events.next(&mut window) {
        window.handle_event(&event);
        if let Some(w_h) = event.resize_args() {
            println!("{:?} {:?}",w_h[0], w_h[1]);
            solver.suggest_value(window_width, w_h[0] as f64).unwrap();
            solver.suggest_value(window_height, w_h[1] as f64).unwrap();
        }

        window.draw_2d(&event, |c, g| {
            clear([0.8, 0.8, 0.8, 1.0], g);
            box1.draw(&mut solver, c, g);
            box2.draw(&mut solver, c, g);
        });
    }
}