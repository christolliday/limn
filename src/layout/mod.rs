use std::ops::Drop;

use cassowary::{Variable, Constraint};
use cassowary::WeightedRelation::*;
use cassowary::strength::*;

use util::{Rectangle, Dimensions};
use layout::constraint::ConstraintBuilder;

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
pub struct LayoutUpdate {
    pub edit_vars: Vec<EditVariable>,
    pub constraints: Vec<Constraint>,
}
impl LayoutUpdate {
    pub fn new(edit_vars: Vec<EditVariable>, constraints: Vec<Constraint>) -> Self {
        LayoutUpdate {
            edit_vars: edit_vars,
            constraints: constraints,
        }
    }
}

pub struct LayoutBuilder {
    pub vars: LayoutVars,
    pub edit_vars: Vec<EditVariable>,
    pub constraints: Vec<Constraint>,
}
impl LayoutBuilder {
    pub fn new() -> Self {
        let vars = LayoutVars::new();
        let mut constraints = Vec::new();
        // always enforce that width is positive
        constraints.push(vars.right | GE(REQUIRED) | vars.left);
        constraints.push(vars.bottom | GE(REQUIRED) | vars.top);
        LayoutBuilder {
            vars: vars,
            edit_vars: Vec::new(),
            constraints: constraints,
        }
    }
    pub fn from(vars: LayoutVars) -> Self {
        LayoutBuilder {
            vars: vars,
            edit_vars: Vec::new(),
            constraints: Vec::new(),
        }
    }
    pub fn edit_left(&mut self) -> VariableEditable {
        let var = self.vars.left;
        VariableEditable::new(self, var)
    }
    pub fn edit_top(&mut self) -> VariableEditable {
        let var = self.vars.top;
        VariableEditable::new(self, var)
    }
    pub fn edit_right(&mut self) -> VariableEditable {
        let var = self.vars.right;
        VariableEditable::new(self, var)
    }
    pub fn edit_bottom(&mut self) -> VariableEditable {
        let var = self.vars.bottom;
        VariableEditable::new(self, var)
    }
    pub fn add<B: ConstraintBuilder>(&mut self, builder: B) {
        let constraints = builder.build(self);
        self.constraints.extend(constraints);
    }
}

pub struct VariableEditable<'a> {
    pub builder: &'a mut LayoutBuilder,
    var: Variable,
    val: f64,
    strength: f64,
}
impl<'a> VariableEditable<'a> {
    pub fn new(builder: &'a mut LayoutBuilder, var: Variable) -> Self {
        VariableEditable {
            builder: builder,
            var: var,
            val: 0.0,
            strength: STRONG,
        }
    }
    pub fn strength(mut self, strength: f64) -> Self {
        self.strength = strength;
        self
    }
    pub fn set(mut self, val: f64) -> Self {
        self.val = val;
        self
    }
}
impl<'a> Drop for VariableEditable<'a> {
    fn drop(&mut self) {
        let edit_var = EditVariable::new(&self);
        self.builder.edit_vars.push(edit_var);
    }
}

pub struct EditVariable {
    var: Variable,
    val: f64,
    strength: f64,
}
impl EditVariable {
    fn new(editable: &VariableEditable) -> Self {
        EditVariable {
            var: editable.var,
            val: editable.val,
            strength: editable.strength,
        }
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

#[macro_export]
macro_rules! layout {
    ($widget:ident: $($func:expr),*) => {
        layout!($widget: $($func, )*);
    };
    ($widget:ident: $($func:expr,)*) => {
        {
            $(
                $widget.layout().add($func);
            )*
        }
    };
}

lazy_static! {
    pub static ref LAYOUT: LayoutVars = LayoutVars::new();
}

pub mod solver;
pub mod container;
pub mod constraint;
pub mod linear_layout;
