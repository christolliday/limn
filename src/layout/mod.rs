pub mod solver;

use std::ops::Drop;

use cassowary::{Variable, Constraint, Expression};
use cassowary::WeightedRelation::*;
use cassowary::strength::*;

use util::{Point, Rectangle, Dimensions, Scalar};

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

pub trait LayoutRef {
    fn layout_ref(&self) -> &LayoutVars;
}

impl<'a> LayoutRef for &'a mut LayoutBuilder {
    fn layout_ref(&self) -> &LayoutVars {
        &self.vars
    }
}
impl LayoutRef for LayoutBuilder {
    fn layout_ref(&self) -> &LayoutVars {
        &self.vars
    }
}
impl LayoutRef for LayoutVars {
    fn layout_ref(&self) -> &LayoutVars {
        self
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
    pub fn match_layout<T: LayoutRef>(&mut self, widget: &T) -> WidgetConstraint {
        let widget = widget.layout_ref();
        let constraints = vec![
            self.vars.left | EQ(REQUIRED) | widget.left,
            self.vars.right | EQ(REQUIRED) | widget.right,
            self.vars.top | EQ(REQUIRED) | widget.top,
            self.vars.bottom | EQ(REQUIRED) | widget.bottom,
        ];
        WidgetConstraint::new_set(self, constraints)
    }
    pub fn match_width<T: LayoutRef>(&mut self, widget: &T) -> WidgetConstraint {
        let widget = widget.layout_ref();
        let constraint = self.vars.right - self.vars.left | EQ(REQUIRED) | widget.right - widget.left;
        WidgetConstraint::new(self, constraint)
    }
    pub fn match_height<T: LayoutRef>(&mut self, widget: &T) -> WidgetConstraint {
        let widget = widget.layout_ref();
        let constraint = self.vars.bottom - self.vars.top | EQ(REQUIRED) | widget.bottom - widget.top;
        WidgetConstraint::new(self, constraint)
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
        self.shrink_horizontal();
        self.shrink_vertical();
    }
    pub fn shrink_horizontal(&mut self) {
        self.width(0.0).strength(WEAK);
    }
    pub fn shrink_vertical(&mut self) {
        self.height(0.0).strength(WEAK);
    }
    pub fn top_left(&mut self, point: Point) -> WidgetConstraint {
        let constraints = vec![
            self.vars.left | EQ(REQUIRED) | point.x,
            self.vars.top | EQ(REQUIRED) | point.y,
        ];
        WidgetConstraint::new_set(self, constraints)
    }
    pub fn center<T: LayoutRef>(&mut self, widget: &T) -> WidgetConstraint {
        let widget = widget.layout_ref();
        let constraints = vec![
            self.vars.left - widget.left | EQ(REQUIRED) | widget.right - self.vars.right,
            self.vars.top - widget.top | EQ(REQUIRED) | widget.bottom - self.vars.bottom,
        ];
        WidgetConstraint::new_set(self, constraints)
    }
    pub fn center_horizontal<T: LayoutRef>(&mut self, widget: &T) -> WidgetConstraint {
        let widget = widget.layout_ref();
        let constraint = self.vars.left - widget.left | EQ(REQUIRED) | widget.right - self.vars.right;
        WidgetConstraint::new(self, constraint)
    }
    pub fn center_vertical<T: LayoutRef>(&mut self, widget: &T) -> WidgetConstraint {
        let widget = widget.layout_ref();
        let constraint = self.vars.top - widget.top | EQ(REQUIRED) | widget.bottom - self.vars.bottom;
        WidgetConstraint::new(self, constraint)
    }

    pub fn align_top<T: LayoutRef>(&mut self, widget: &T) -> PaddableConstraint {
        let widget = widget.layout_ref();
        let constraint = self.vars.top - widget.top | EQ(REQUIRED) | 0.0;
        PaddableConstraint::new(self, constraint)
    }
    pub fn align_bottom<T: LayoutRef>(&mut self, widget: &T) -> PaddableConstraint {
        let widget = widget.layout_ref();
        let constraint = widget.bottom - self.vars.bottom | EQ(REQUIRED) | 0.0;
        PaddableConstraint::new(self, constraint)
    }
    pub fn align_left<T: LayoutRef>(&mut self, widget: &T) -> PaddableConstraint {
        let widget = widget.layout_ref();
        let constraint = self.vars.left - widget.left | EQ(REQUIRED) | 0.0;
        PaddableConstraint::new(self, constraint)
    }
    pub fn align_right<T: LayoutRef>(&mut self, widget: &T) -> PaddableConstraint {
        let widget = widget.layout_ref();
        let constraint = widget.right - self.vars.right | EQ(REQUIRED) | 0.0;
        PaddableConstraint::new(self, constraint)
    }

    pub fn above<T: LayoutRef>(&mut self, widget: &T) -> PaddableConstraint {
        let widget = widget.layout_ref();
        let constraint = self.vars.bottom - widget.top | GE(REQUIRED) | 0.0;
        PaddableConstraint::new(self, constraint)
    }
    pub fn below<T: LayoutRef>(&mut self, widget: &T) -> PaddableConstraint {
        let widget = widget.layout_ref();
        let constraint = self.vars.top - widget.bottom | GE(REQUIRED) | 0.0;
        PaddableConstraint::new(self, constraint)
    }
    pub fn to_left_of<T: LayoutRef>(&mut self, widget: &T) -> PaddableConstraint {
        let widget = widget.layout_ref();
        let constraint = widget.left - self.vars.right | GE(REQUIRED) | 0.0;
        PaddableConstraint::new(self, constraint)
    }
    pub fn to_right_of<T: LayoutRef>(&mut self, widget: &T) -> PaddableConstraint {
        let widget = widget.layout_ref();
        let constraint = self.vars.left - widget.right | GE(REQUIRED) | 0.0;
        PaddableConstraint::new(self, constraint)
    }

    pub fn bound_left<T: LayoutRef>(&mut self, outer: &T) -> PaddableConstraint {
        let outer = outer.layout_ref();
        let constraint = self.vars.left - outer.left | GE(REQUIRED) | 0.0;
        PaddableConstraint::new(self, constraint)
    }
    pub fn bound_top<T: LayoutRef>(&mut self, outer: &T) -> PaddableConstraint {
        let outer = outer.layout_ref();
        let constraint = self.vars.top - outer.top | GE(REQUIRED) | 0.0;
        PaddableConstraint::new(self, constraint)
    }
    pub fn bound_right<T: LayoutRef>(&mut self, outer: &T) -> PaddableConstraint {
        let outer = outer.layout_ref();
        let constraint = outer.right - self.vars.right | GE(REQUIRED) | 0.0;
        PaddableConstraint::new(self, constraint)
    }
    pub fn bound_bottom<T: LayoutRef>(&mut self, outer: &T) -> PaddableConstraint {
        let outer = outer.layout_ref();
        let constraint = outer.bottom - self.vars.bottom | GE(REQUIRED) | 0.0;
        PaddableConstraint::new(self, constraint)
    }

    pub fn bound_by<T: LayoutRef>(&mut self, outer: &T) -> PaddableConstraint {
        let outer = outer.layout_ref();
        let constraints = vec![
            self.vars.left - outer.left | GE(REQUIRED) | 0.0,
            self.vars.top - outer.top | GE(REQUIRED) | 0.0,
            outer.right - self.vars.right | GE(REQUIRED) | 0.0,
            outer.bottom - self.vars.bottom | GE(REQUIRED) | 0.0,
        ];
        PaddableConstraint::new_set(self, constraints)
    }

    pub fn scroll_parent<T: LayoutRef>(&mut self, inner: &T) {
        let inner = inner.layout_ref();
        self.constraints.push(inner.left | LE(REQUIRED) | self.vars.left);
        self.constraints.push(inner.top | LE(REQUIRED) | self.vars.top);
        // STRONG not REQUIRED because not satisfiable if layout is smaller than it's parent
        self.constraints.push(inner.right | GE(STRONG) | self.vars.right);
        self.constraints.push(inner.bottom | GE(STRONG) | self.vars.bottom);
    }
}

#[macro_export]
macro_rules! layout {
    (parent: $parent:expr, $($type:path: $val:expr),*) => {
        {
            use $crate::widget::style::Value;
            let mut style = $parent.clone();
            $(
                style.push($type(Value::from($val)));
            )*
            style
        }
    };
    ($($type:path: $val:expr),*) => {
        {
            use $crate::widget::style::Value;
            vec![
                $(
                    $type(Value::from($val)),
                )*
            ]
        }
    };
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