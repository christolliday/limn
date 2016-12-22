use cassowary::{Solver, Variable, Constraint};
use cassowary::WeightedRelation::*;
use cassowary::strength::*;

use super::super::util::*;

pub struct WidgetLayout {
    pub left: Variable,
    pub right: Variable,
    pub top: Variable,
    pub bottom: Variable,
    pub constraints: Vec<Constraint>,
}
impl WidgetLayout {
    pub fn new() -> Self {
        WidgetLayout {
            left: Variable::new(),
            right: Variable::new(),
            top: Variable::new(),
            bottom: Variable::new(),
            constraints: Vec::new(),
        }
    }
    pub fn bounds(&self, solver: &mut Solver) -> Rectangle {
        Rectangle {
            left: solver.get_value(self.left),
            top: solver.get_value(self.top),
            width: solver.get_value(self.right) - solver.get_value(self.left),
            height: solver.get_value(self.bottom) - solver.get_value(self.top),
        }
    }
    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }
    pub fn add_constraints(&mut self, constraints: &[Constraint]) {
        self.constraints.extend_from_slice(constraints);
    }
    pub fn width(&mut self, width: Scalar, strength: f64) {
        self.constraints.push(self.right - self.left | EQ(strength) | width)
    }
    pub fn height(&mut self, height: Scalar, strength: f64) {
        self.constraints.push(self.bottom - self.top | EQ(strength) | height)
    }
    pub fn bound_by(&mut self, layout: &WidgetLayout) {
        let constraints = [layout.left | GE(REQUIRED) | self.left,
                           layout.top | GE(REQUIRED) | self.top,
                           layout.right | LE(REQUIRED) | self.right,
                           layout.bottom | LE(REQUIRED) | self.bottom];
        self.add_constraints(&constraints);
    }
}