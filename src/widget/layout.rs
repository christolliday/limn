use cassowary::{Solver, Variable, Constraint};
use cassowary::WeightedRelation::*;
use cassowary::strength::*;

use util::{Point, Rectangle, Dimensions, Scalar};

#[derive(Copy, Clone)]
pub enum Orientation {
    Horizontal,
    Vertical,
}
pub struct LinearLayout {
    pub orientation: Orientation,
    pub end: Variable,
}
impl LinearLayout {
    pub fn new(orientation: Orientation, parent: &LayoutBuilder) -> Self {
        LinearLayout {
            orientation: orientation,
            end: LinearLayout::beginning(orientation, &parent.vars),
        }
    }
    pub fn beginning(orientation: Orientation, layout: &LayoutVars) -> Variable {
        match orientation {
            Orientation::Horizontal => layout.left,
            Orientation::Vertical => layout.top,
        }
    }
    pub fn ending(orientation: Orientation, layout: &LayoutVars) -> Variable {
        match orientation {
            Orientation::Horizontal => layout.right,
            Orientation::Vertical => layout.bottom,
        }
    }
    pub fn add_widget(&mut self, widget_layout: &mut LayoutBuilder) {
        let constraint = LinearLayout::beginning(self.orientation, &widget_layout.vars) |
                         GE(REQUIRED) | self.end;
        self.end = LinearLayout::ending(self.orientation, &widget_layout.vars);
        widget_layout.add_constraint(constraint);
    }
}

/*
constraint types, for declarative layout
pub enum ConstraintType {
    // EQ, StrDefault: WEAK
    ShrinkWidth,
    ShrinkHeight,
    // EQ, StrDefault: WEAK,
    GrowWidth,
    GrowHeight,
    // EQ, Args(Widget), StrDefault: REQ
    MatchWidth,
    MatchHeight,
    // GE, StrDefault: REQ
    MinWidth,
    MinHeight,
    // GE, StrDefault: REQ,
    MaxWidth,
    MaxHeight,
    // EQ, Args(Widget), StrDefault: REQ
    CenterHorizontal,
    CenterVertical,
    // EQ, Args(Widget, Option<padding>), StrDefault: REQ
    AlignTop,
    AlignBottom,
    AlignLeft,
    AlignRight,
    // GE, Args(Widget, Option<padding>), StrDefault: REQ
    BoundTop,
    BoundBottom,
    BoundLeft,
    BoundRight,
    // GE, Args(Widget, Option<padding>), StrDefault: REQ
    Above,
    Below,
    ToRightOf,
    ToLeftOf,
}
*/

pub struct LayoutVars {
    pub left: Variable,
    pub top: Variable,
    pub right: Variable,
    pub bottom: Variable,
}
impl LayoutVars {
    pub fn new() -> Self {
        LayoutVars {
            left: Variable::new(),
            top: Variable::new(),
            right: Variable::new(),
            bottom: Variable::new(),
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
    pub fn get_dims(&self, solver: &mut Solver) -> Dimensions {
        let bounds = self.bounds(solver);
        Dimensions {
            width: bounds.width,
            height: bounds.height,
        }
    }
}

pub struct LayoutBuilder {
    pub vars: LayoutVars,
    pub constraints: Vec<Constraint>,
}
impl LayoutBuilder {
    pub fn new() -> Self {
        LayoutBuilder {
            vars: LayoutVars::new(),
            constraints: Vec::new(),
        }
    }
    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }
    pub fn add_constraints(&mut self, constraints: &[Constraint]) {
        self.constraints.extend_from_slice(constraints);
    }
    pub fn build(self) -> (LayoutVars, Vec<Constraint>) {
        (self.vars, self.constraints)
    }
    pub fn match_layout(&mut self, layout: &LayoutVars) {
        self.match_width(layout);
        self.match_height(layout);
    }
    pub fn match_width(&mut self, layout: &LayoutVars) {
        self.constraints.push(self.vars.left | EQ(REQUIRED) | layout.left);
        self.constraints.push(self.vars.right | EQ(REQUIRED) | layout.right);
    }
    pub fn match_height(&mut self, layout: &LayoutVars) {
        self.constraints.push(self.vars.top | EQ(REQUIRED) | layout.top);
        self.constraints.push(self.vars.bottom | EQ(REQUIRED) | layout.bottom);
    }
    pub fn width(&mut self, width: Scalar) {
        self.constraints.push(self.vars.right - self.vars.left | EQ(REQUIRED) | width)
    }
    pub fn height(&mut self, height: Scalar) {
        self.constraints.push(self.vars.bottom - self.vars.top | EQ(REQUIRED) | height)
    }
    pub fn dimensions(&mut self, dimensions: Dimensions) {
        self.width(dimensions.width);
        self.height(dimensions.height);
    }
    pub fn minimize(&mut self) {
        self.width_strength(0.0, WEAK);
        self.height_strength(0.0, WEAK);
    }
    pub fn maximize(&mut self) {
        self.width_strength(10000.0, WEAK);
        self.height_strength(10000.0, WEAK);
    }
    pub fn width_strength(&mut self, width: Scalar, strength: f64) {
        self.constraints.push(self.vars.right - self.vars.left | EQ(strength) | width)
    }
    pub fn height_strength(&mut self, height: Scalar, strength: f64) {
        self.constraints.push(self.vars.bottom - self.vars.top | EQ(strength) | height)
    }
    pub fn top_left(&mut self, point: Point, strength: Option<f64>) {
        self.constraints.push(self.vars.left | EQ(strength.unwrap_or(REQUIRED)) | point.x);
        self.constraints.push(self.vars.top | EQ(strength.unwrap_or(REQUIRED)) | point.y);
    }
    pub fn center(&mut self, layout: &LayoutVars) {
        self.center_horizontal(layout);
        self.center_vertical(layout);
    }
    pub fn center_horizontal(&mut self, layout: &LayoutVars) {
        self.constraints.push(self.vars.left - layout.left | EQ(REQUIRED) | layout.right - self.vars.right);
    }
    pub fn center_vertical(&mut self, layout: &LayoutVars) {
        self.constraints.push(self.vars.top - layout.top | EQ(REQUIRED) | layout.bottom - self.vars.bottom);
    }

    pub fn align_top(&mut self, layout: &LayoutVars, padding: Option<f64>) {
        self.constraints.push(self.vars.top - layout.top | EQ(REQUIRED) | padding.unwrap_or(0.0));
    }
    pub fn align_bottom(&mut self, layout: &LayoutVars, padding: Option<f64>) {
        self.constraints.push(layout.bottom - self.vars.bottom | EQ(REQUIRED) | padding.unwrap_or(0.0));
    }
    pub fn align_left(&mut self, layout: &LayoutVars, padding: Option<f64>) {
        self.constraints.push(self.vars.left - layout.left | EQ(REQUIRED) | padding.unwrap_or(0.0));
    }
    pub fn align_right(&mut self, layout: &LayoutVars, padding: Option<f64>) {
        self.constraints.push(layout.right - self.vars.right | EQ(REQUIRED) | padding.unwrap_or(0.0));
    }
    
    pub fn above(&mut self, layout: &LayoutVars, padding: Option<f64>) {
        self.constraints.push(self.vars.bottom - layout.top | GE(REQUIRED) | padding.unwrap_or(0.0));
    }
    pub fn below(&mut self, layout: &LayoutVars, padding: Option<f64>) {
        self.constraints.push(layout.bottom - self.vars.top | GE(REQUIRED) | padding.unwrap_or(0.0));
    }
    pub fn to_left_of(&mut self, layout: &LayoutVars, padding: Option<f64>) {
        self.constraints.push(layout.left - self.vars.right | GE(REQUIRED) | padding.unwrap_or(0.0));
    }
    pub fn to_right_of(&mut self, layout: &LayoutVars, padding: Option<f64>) {
        self.constraints.push(self.vars.left - layout.right | GE(REQUIRED) | padding.unwrap_or(0.0));
    }

    pub fn bound_left(&mut self, layout: &LayoutVars, padding: Option<f64>) {
        self.constraints.push(self.vars.left - layout.left | GE(REQUIRED) | padding.unwrap_or(0.0));
    }
    pub fn bound_top(&mut self, layout: &LayoutVars, padding: Option<f64>) {
        self.constraints.push(self.vars.top - layout.top | GE(REQUIRED) | padding.unwrap_or(0.0));
    }
    pub fn bound_right(&mut self, layout: &LayoutVars, padding: Option<f64>) {
        self.constraints.push(layout.right - self.vars.right| GE(REQUIRED) | padding.unwrap_or(0.0));
    }
    pub fn bound_bottom(&mut self, layout: &LayoutVars, padding: Option<f64>) {
        self.constraints.push(layout.bottom - self.vars.bottom | GE(REQUIRED) | padding.unwrap_or(0.0));
    }

    pub fn bound_by(&mut self, layout: &LayoutVars, padding: Option<f64>) {
        self.bound_left(layout, padding);
        self.bound_top(layout, padding);
        self.bound_right(layout, padding);
        self.bound_bottom(layout, padding);
    }

    pub fn scroll_inside(&mut self, layout: &LayoutVars) {
        let constraints = [self.vars.left | LE(REQUIRED) | layout.left,
                           self.vars.top | LE(REQUIRED) | layout.top,
                           // STRONG not REQUIRED because not satisfiable if layout is smaller than it's parent
                           self.vars.right | GE(STRONG) | layout.right,
                           self.vars.bottom | GE(STRONG) | layout.bottom];
        self.add_constraints(&constraints);
    }
}