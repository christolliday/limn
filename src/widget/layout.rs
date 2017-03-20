use std::ops::Drop;

use cassowary::{Variable, Constraint, Expression};
use cassowary::WeightedRelation::*;
use cassowary::strength::*;

use util::{Point, Rectangle, Dimensions, Scalar};

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

#[derive(Clone)]
pub struct LayoutVars {
    pub left: Variable,
    pub top: Variable,
    pub right: Variable,
    pub bottom: Variable,

    left_val: f64,
    top_val: f64,
    right_val: f64,
    bottom_val: f64,
}
impl LayoutVars {
    pub fn new() -> Self {
        LayoutVars {
            left: Variable::new(),
            top: Variable::new(),
            right: Variable::new(),
            bottom: Variable::new(),
            left_val: 0.0,
            top_val: 0.0,
            right_val: 0.0,
            bottom_val: 0.0,
        }
    }
    pub fn update_val(&mut self, var: Variable, value: f64) {
        if var == self.left {
            self.left_val = value;
        } else if var == self.top {
            self.top_val = value;
        } else if var == self.right {
            self.right_val = value;
        } else if var == self.bottom {
            self.bottom_val = value;
        } else {
            panic!();
        }
    }
    pub fn bounds(&self) -> Rectangle {
        Rectangle {
            left: self.left_val,
            top: self.top_val,
            width: self.right_val - self.left_val,
            height: self.bottom_val - self.top_val, 
        }
    }
    pub fn get_dims(&self) -> Dimensions {
        Dimensions {
            width: self.right_val - self.left_val,
            height: self.bottom_val - self.top_val,
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
    pub fn from(vars: LayoutVars) -> Self {
        LayoutBuilder {
            vars: vars,
            constraints: Vec::new(),
        }
    }
    pub fn match_layout(&mut self, widget: &LayoutVars) -> WidgetConstraint {
        let constraints = vec![
            self.vars.left | EQ(REQUIRED) | widget.left,
            self.vars.right | EQ(REQUIRED) | widget.right,
            self.vars.top | EQ(REQUIRED) | widget.top,
            self.vars.bottom | EQ(REQUIRED) | widget.bottom,
        ];
        WidgetConstraint::new_set(self, constraints)
    }
    pub fn match_width(&mut self, widget: &LayoutVars) -> WidgetConstraint {
        let constraints = vec![
            self.vars.left | EQ(REQUIRED) | widget.left,
            self.vars.right | EQ(REQUIRED) | widget.right,
        ];
        WidgetConstraint::new_set(self, constraints)
    }
    pub fn match_height(&mut self, widget: &LayoutVars) -> WidgetConstraint {
        let constraints = vec![
            self.vars.top | EQ(REQUIRED) | widget.top,
            self.vars.bottom | EQ(REQUIRED) | widget.bottom,
        ];
        WidgetConstraint::new_set(self, constraints)
    }
    pub fn width(&mut self, width: Scalar) -> WidgetConstraint {
        let constraint = self.vars.right - self.vars.left | EQ(REQUIRED) | width;
        WidgetConstraint::new(self, constraint)
    }
    pub fn height(&mut self, height: Scalar) -> WidgetConstraint {
        let constraint = self.vars.bottom - self.vars.top | EQ(REQUIRED) | height;
        WidgetConstraint::new(self, constraint)
    }
    pub fn min_width(&mut self, width: Scalar) -> WidgetConstraint {
        let constraint = self.vars.right - self.vars.left | GE(REQUIRED) | width;
        WidgetConstraint::new(self, constraint)
    }
    pub fn min_height(&mut self, height: Scalar) -> WidgetConstraint {
        let constraint = self.vars.bottom - self.vars.top | GE(REQUIRED) | height;
        WidgetConstraint::new(self, constraint)
    }
    pub fn dimensions(&mut self, dimensions: Dimensions) -> WidgetConstraint {
        let constraints = vec![
            self.vars.right - self.vars.left | EQ(REQUIRED) | dimensions.width,
            self.vars.bottom - self.vars.top | EQ(REQUIRED) | dimensions.height,
        ];
        WidgetConstraint::new_set(self, constraints)
    }
    pub fn min_dimensions(&mut self, dimensions: Dimensions) -> WidgetConstraint {
        let constraints = vec![
            self.vars.right - self.vars.left | GE(REQUIRED) | dimensions.width,
            self.vars.bottom - self.vars.top | GE(REQUIRED) | dimensions.height,
        ];
        WidgetConstraint::new_set(self, constraints)
    }
    pub fn shrink(&mut self) {
        self.width(0.0).strength(WEAK);
        self.height(0.0).strength(WEAK);
    }
    pub fn top_left(&mut self, point: Point, strength: Option<f64>) -> WidgetConstraint {
        let constraints = vec![
            self.vars.left | EQ(strength.unwrap_or(REQUIRED)) | point.x,
            self.vars.top | EQ(strength.unwrap_or(REQUIRED)) | point.y,
        ];
        WidgetConstraint::new_set(self, constraints)
    }
    pub fn center(&mut self, widget: &LayoutVars) -> WidgetConstraint {
        let constraints = vec![
            self.vars.left - widget.left | EQ(REQUIRED) | widget.right - self.vars.right,
            self.vars.top - widget.top | EQ(REQUIRED) | widget.bottom - self.vars.bottom,
        ];
        WidgetConstraint::new_set(self, constraints)
    }
    pub fn center_horizontal(&mut self, widget: &LayoutVars) -> WidgetConstraint {
        let constraint = self.vars.left - widget.left | EQ(REQUIRED) | widget.right - self.vars.right;
        WidgetConstraint::new(self, constraint)
    }
    pub fn center_vertical(&mut self, widget: &LayoutVars) -> WidgetConstraint {
        let constraint = self.vars.top - widget.top | EQ(REQUIRED) | widget.bottom - self.vars.bottom;
        WidgetConstraint::new(self, constraint)
    }

    pub fn align_top(&mut self, widget: &LayoutVars) -> PaddableConstraint {
        let constraint = self.vars.top - widget.top | EQ(REQUIRED) | 0.0;
        PaddableConstraint::new(self, constraint)
    }
    pub fn align_bottom(&mut self, widget: &LayoutVars) -> PaddableConstraint {
        let constraint = widget.bottom - self.vars.bottom | EQ(REQUIRED) | 0.0;
        PaddableConstraint::new(self, constraint)
    }
    pub fn align_left(&mut self, widget: &LayoutVars) -> PaddableConstraint {
        let constraint = self.vars.left - widget.left | EQ(REQUIRED) | 0.0;
        PaddableConstraint::new(self, constraint)
    }
    pub fn align_right(&mut self, widget: &LayoutVars) -> PaddableConstraint {
        let constraint = widget.right - self.vars.right | EQ(REQUIRED) | 0.0;
        PaddableConstraint::new(self, constraint)
    }

    pub fn above(&mut self, widget: &LayoutVars) -> PaddableConstraint {
        let constraint = self.vars.bottom - widget.top | GE(REQUIRED) | 0.0;
        PaddableConstraint::new(self, constraint)
    }
    pub fn below(&mut self, widget: &LayoutVars) -> PaddableConstraint {
        let constraint = self.vars.top - widget.bottom | GE(REQUIRED) | 0.0;
        PaddableConstraint::new(self, constraint)
    }
    pub fn to_left_of(&mut self, widget: &LayoutVars) -> PaddableConstraint {
        let constraint = widget.left - self.vars.right | GE(REQUIRED) | 0.0;
        PaddableConstraint::new(self, constraint)
    }
    pub fn to_right_of(&mut self, widget: &LayoutVars) -> PaddableConstraint {
        let constraint = self.vars.left - widget.right | GE(REQUIRED) | 0.0;
        PaddableConstraint::new(self, constraint)
    }

    pub fn bound_left(&mut self, widget: &LayoutVars) -> PaddableConstraint {
        let constraint = self.vars.left - widget.left | GE(REQUIRED) | 0.0;
        PaddableConstraint::new(self, constraint)
    }
    pub fn bound_top(&mut self, widget: &LayoutVars) -> PaddableConstraint {
        let constraint = self.vars.top - widget.top | GE(REQUIRED) | 0.0;
        PaddableConstraint::new(self, constraint)
    }
    pub fn bound_right(&mut self, widget: &LayoutVars) -> PaddableConstraint {
        let constraint = widget.right - self.vars.right | GE(REQUIRED) | 0.0;
        PaddableConstraint::new(self, constraint)
    }
    pub fn bound_bottom(&mut self, widget: &LayoutVars) -> PaddableConstraint {
        let constraint = widget.bottom - self.vars.bottom | GE(REQUIRED) | 0.0;
        PaddableConstraint::new(self, constraint)
    }

    pub fn bound_by(&mut self, widget: &LayoutVars) -> PaddableConstraint {
        let constraints = vec![
            self.vars.left - widget.left | GE(REQUIRED) | 0.0,
            self.vars.top - widget.top | GE(REQUIRED) | 0.0,
            widget.right - self.vars.right | GE(REQUIRED) | 0.0,
            widget.bottom - self.vars.bottom | GE(REQUIRED) | 0.0,
        ];
        PaddableConstraint::new_set(self, constraints)
    }

    pub fn scroll_inside(&mut self, outer: &LayoutVars) {
        self.constraints.push(self.vars.left | LE(REQUIRED) | outer.left);
        self.constraints.push(self.vars.top | LE(REQUIRED) | outer.top);
        // STRONG not REQUIRED because not satisfiable if layout is smaller than it's parent
        self.constraints.push(self.vars.right | GE(STRONG) | outer.right);
        self.constraints.push(self.vars.bottom | GE(STRONG) | outer.bottom);
    }
    pub fn scroll_parent(&mut self, inner: &LayoutVars) {
        self.constraints.push(inner.left | LE(REQUIRED) | self.vars.left);
        self.constraints.push(inner.top | LE(REQUIRED) | self.vars.top);
        // STRONG not REQUIRED because not satisfiable if layout is smaller than it's parent
        self.constraints.push(inner.right | GE(STRONG) | self.vars.right);
        self.constraints.push(inner.bottom | GE(STRONG) | self.vars.bottom);
    }
}

pub struct WidgetConstraint<'a> {
    pub builder: &'a mut LayoutBuilder,
    pub constraints: Vec<Constraint>,
}
impl<'a> WidgetConstraint<'a> {
    pub fn new(builder: &'a mut LayoutBuilder, constraint: Constraint) -> Self {
        WidgetConstraint {
            builder: builder,
            constraints: vec![constraint],
        }
    }
    pub fn new_set(builder: &'a mut LayoutBuilder, constraints: Vec<Constraint>) -> Self {
        WidgetConstraint {
            builder: builder,
            constraints: constraints,
        }
    }
    pub fn strength(mut self, strength: f64) -> Self {
        self.constraints = change_strength(&self.constraints, strength);
        self
    }
}
pub struct PaddableConstraint<'a> {
    pub builder: &'a mut LayoutBuilder,
    pub constraints: Vec<Constraint>,
}

impl<'a> PaddableConstraint<'a> {
    pub fn new(builder: &'a mut LayoutBuilder, constraint: Constraint) -> Self {
        PaddableConstraint {
            builder: builder,
            constraints: vec![constraint],
        }
    }
    pub fn new_set(builder: &'a mut LayoutBuilder, constraints: Vec<Constraint>) -> Self {
        PaddableConstraint {
            builder: builder,
            constraints: constraints,
        }
    }
    pub fn strength(mut self, strength: f64) -> Self {
        self.constraints = change_strength(&self.constraints, strength);
        self
    }
    pub fn padding(mut self, padding: Scalar) -> Self {
        let mut new_constraints = Vec::new();
        for cons in &self.constraints {
            // replace constant in existing constraint, with padding value, negative because on the same side as the terms
            let expression = Expression::new(cons.0.expression.terms.clone(), -padding);
            let cons = Constraint::new(expression, cons.0.op, cons.0.strength);
            new_constraints.push(cons);
        }
        self.constraints = new_constraints;
        self
    }
}

impl<'a> Drop for PaddableConstraint<'a> {
    fn drop(&mut self) {
        self.builder.constraints.extend(self.constraints.clone());
    }
}
impl<'a> Drop for WidgetConstraint<'a> {
    fn drop(&mut self) {
        self.builder.constraints.extend(self.constraints.clone());
    }
}

pub fn change_strength(constraints: &Vec<Constraint>, strength: f64) -> Vec<Constraint> {
    let mut new_constraints = Vec::new();
    for cons in constraints {
        let cons = Constraint::new(cons.0.expression.clone(), cons.0.op, strength);
        new_constraints.push(cons);
    }
    new_constraints
}