use std::ops::Drop;

use cassowary::{Variable, Constraint};
use cassowary::WeightedRelation::*;
use cassowary::strength::*;

use util::Rect;
use layout::constraint::ConstraintBuilder;

#[derive(Clone)]
pub struct LayoutVars {
    pub left: Variable,
    pub top: Variable,
    pub right: Variable,
    pub bottom: Variable,
    pub width: Variable,
    pub height: Variable,
}
impl LayoutVars {
    pub fn new() -> Self {
        LayoutVars {
            left: Variable::new(),
            top: Variable::new(),
            right: Variable::new(),
            bottom: Variable::new(),
            width: Variable::new(),
            height: Variable::new(),
        }
    }
    pub fn update_bounds(&self, var: Variable, value: f64, rect: &mut Rect) {
        if var == self.left {
            rect.origin.x = value;
        } else if var == self.top {
            rect.origin.y = value;
        } else if var == self.width {
            rect.size.width = value;
        } else if var == self.height {
            rect.size.height = value;
        } else {
            panic!();
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
    pub edit_vars: Vec<EditVariable>,
    pub constraints: Vec<Constraint>,
}
impl LayoutBuilder {
    pub fn new() -> Self {
        let vars = LayoutVars::new();
        let mut constraints = Vec::new();
        constraints.push(vars.right - vars.left| EQ(REQUIRED) | vars.width);
        constraints.push(vars.bottom - vars.top | EQ(REQUIRED) | vars.height);
        // temporarily disabling this, as it tends to cause width/height to snap to 0
        //constraints.push(vars.width | GE(REQUIRED) | 0.0);
        //constraints.push(vars.height | GE(REQUIRED) | 0.0);
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

pub use self::solver::LimnSolver;
