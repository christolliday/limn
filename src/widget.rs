
use backend::gfx::G2d;
use graphics::*;

use graphics::types::{Color, Scalar};
use super::{Point, Dimensions, EventListener};

use cassowary::{ Solver, Variable, Constraint };
use cassowary::WeightedRelation::*;
use cassowary::strength::{ WEAK, MEDIUM, STRONG, REQUIRED };

fn point_inside_rect(point: Point, rect: types::Rectangle) -> bool {
    point.x > rect[0] && point.y > rect[1] && point.x < rect[0] + rect[2] && point.y < rect[1] + rect[3]
}
fn point_inside_ellipse(point: Point, center: Point, radius: Dimensions) -> bool {
    (point.x - center.x).powi(2) / radius.width.powi(2) + (point.y - center.y).powi(2) / radius.height.powi(2) <= 1.0
}
pub trait WidgetDrawable {
    fn draw(&self, bounds: types::Rectangle, c: Context, g: &mut G2d);
    fn is_mouse_over(&self, mouse: Point, bounds: types::Rectangle) -> bool {
        point_inside_rect(mouse, bounds)
    }
}

pub struct RectDrawable {
    background: Color,
}
impl RectDrawable {
    pub fn new(color: [f32; 3]) -> Self {
        RectDrawable { background: [color[0], color[1], color[2], 1.0] }
    }
}
impl WidgetDrawable for RectDrawable {
    fn draw(&self, bounds: types::Rectangle, c: Context, g: &mut G2d) {
        Rectangle::new(self.background).draw(bounds, &c.draw_state, c.transform, g);
    }
}

pub struct EllipseDrawable {
    background: Color,
}
impl EllipseDrawable {
    pub fn new(color: [f32; 3]) -> Self {
        EllipseDrawable { background: [color[0], color[1], color[2], 1.0] }
    }
}
impl WidgetDrawable for EllipseDrawable {
    fn draw(&self, bounds: types::Rectangle, c: Context, g: &mut G2d) {
        Ellipse::new(self.background).draw(bounds, &c.draw_state, c.transform, g);
    }
    fn is_mouse_over(&self, mouse: Point, bounds: types::Rectangle) -> bool {
        let radius = Dimensions { width: bounds[2] / 2.0, height: bounds[3] / 2.0 };
        let center = Point { x: bounds[0] + radius.width, y: bounds[1] + radius.height };
        point_inside_ellipse(mouse, center, radius)
    }
}

pub struct WidgetLayout {
    pub left: Variable,
    pub right: Variable,
    pub top: Variable,
    pub bottom: Variable,
    pub constraints: Vec<Constraint>,
}
impl WidgetLayout {
    fn new() -> Self {
        WidgetLayout {
            left: Variable::new(),
            right: Variable::new(),
            top: Variable::new(),
            bottom: Variable::new(),
            constraints: Vec::new(),
        }
    }
    pub fn bounds(&self, solver: &mut Solver) -> types::Rectangle {
        [
            solver.get_value(self.left),
            solver.get_value(self.top),
            solver.get_value(self.right) - solver.get_value(self.left),
            solver.get_value(self.bottom) - solver.get_value(self.top),
        ]
    }
    // layout
    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }
    pub fn add_constraints(&mut self, constraints: &[Constraint]) {
        self.constraints.extend_from_slice(constraints);
    }
    pub fn width(&mut self, width: Scalar, strength: f64) {
        self.constraints.push(self.right - self.left |EQ(strength)| width)
    }
    pub fn height(&mut self, height: Scalar, strength: f64) {
        self.constraints.push(self.bottom - self.top |EQ(strength)| height)
    }
    pub fn bound_by<'a>(&mut self, layout: &WidgetLayout) {
        let constraints = [
            layout.left |GE(REQUIRED)| self.left,
            layout.top |GE(REQUIRED)| self.top,
            layout.right |LE(REQUIRED)| self.right,
            layout.bottom |LE(REQUIRED)| self.bottom,
        ];
        self.add_constraints(&constraints);
    }
}

pub struct Widget<'a> {
    pub drawable: Option<&'a WidgetDrawable>,
    pub layout: WidgetLayout,
    pub listeners: Vec<&'a EventListener>
}

impl<'a> Widget<'a>  {
    pub fn new(drawable: Option<&'a WidgetDrawable>) -> Self {
        Widget {
            drawable: drawable,
            layout: WidgetLayout::new(),
            listeners: Vec::new(),
        }
    }
    pub fn print(&self, solver: &mut Solver) {
        println!("{:?}",
            self.layout.bounds(solver));
    }
    pub fn draw(&self, solver: &mut Solver, c: Context, g: &mut G2d) {
        if let Some(drawable) = self.drawable {
            drawable.draw(self.layout.bounds(solver), c, g);
        }
    }
    pub fn is_mouse_over(&self, solver: &mut Solver, mouse: Point) -> bool {
        let bounds = self.layout.bounds(solver);
        if let Some(drawable) = self.drawable {
            drawable.is_mouse_over(mouse, bounds)
        } else {
            point_inside_rect(mouse, bounds)
        }
    }
}