use cassowary::{Solver, Variable, Constraint};
use cassowary::WeightedRelation::*;
use cassowary::strength::*;

use super::super::util::*;

pub struct WidgetLayout {
    pub left: Variable,
    pub top: Variable,
    pub right: Variable,
    pub bottom: Variable,
    pub scrollable: bool,
    pub constraints: Vec<Constraint>,
}
impl WidgetLayout {
    pub fn new() -> Self {
        WidgetLayout {
            left: Variable::new(),
            top: Variable::new(),
            right: Variable::new(),
            bottom: Variable::new(),
            scrollable: false,
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
    pub fn width(&mut self, width: Scalar) {
        self.constraints.push(self.right - self.left | EQ(REQUIRED) | width)
    }
    pub fn height(&mut self, height: Scalar) {
        self.constraints.push(self.bottom - self.top | EQ(REQUIRED) | height)
    }
    pub fn center(&mut self, layout: &WidgetLayout) {
        let constraints = [self.left - layout.left | EQ(STRONG) | layout.right - self.right,
                           self.top - layout.top | EQ(STRONG) | layout.bottom - self.bottom];
        self.add_constraints(&constraints);
    }
    pub fn bound_by(&mut self, layout: &WidgetLayout) {
        if !layout.scrollable { 
            let constraints = [self.left | GE(REQUIRED) | layout.left,
                            self.top | GE(REQUIRED) | layout.top,
                            self.right | LE(REQUIRED) | layout.right,
                            self.bottom | LE(REQUIRED) | layout.bottom];
            self.add_constraints(&constraints);
        } else {
            let constraints = [self.left | LE(STRONG) | layout.left,
                            self.top | LE(STRONG) | layout.top,
                            self.right | GE(STRONG) | layout.right,
                            self.bottom | GE(STRONG) | layout.bottom];
            self.add_constraints(&constraints);
        }
    }
}
