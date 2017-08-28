use std::sync::Mutex;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;

use cassowary;
use cassowary::strength;
use cassowary::{Variable, Constraint, Expression};

use super::{LayoutId, Layout, LayoutVars};

pub struct LimnSolver {
    pub solver: cassowary::Solver,
    layouts: LayoutManager,
}

impl LimnSolver {
    pub fn new() -> Self {
        LimnSolver {
            solver: cassowary::Solver::new(),
            layouts: LayoutManager::new(),
        }
    }

    pub fn register_widget(&mut self, layout: &mut Layout) {
        self.layouts.register_widget(layout);
    }

    pub fn remove_widget(&mut self, id: LayoutId) {
        self.layouts.layout_names.remove(&id);
        if let Some(layout) = self.layouts.layouts.remove(&id) {
            for var in layout.array().iter() {
                // remove constraints that are relative to this widget from solver
                if let Some(constraint_set) = self.layouts.var_constraints.remove(&var) {
                    for constraint in constraint_set {
                        if self.solver.has_constraint(&constraint) {
                            self.solver.remove_constraint(&constraint).unwrap();
                            // look up other variables that references this constraint,
                            // and remove this constraint from those variables constraint sets
                            if let Some(var_list) = self.layouts.constraint_vars.get(&constraint) {
                                for var in var_list {
                                    if let Some(constraint_set) = self.layouts.var_constraints.get_mut(&var) {
                                        constraint_set.remove(&constraint);
                                    }
                                }
                            }
                        }
                    }
                }
                self.layouts.var_ids.remove(&var);
            }
        }
    }
    // hide/unhide are a simplified way of temporarily removing a layout, by removing
    // only the constraints on that widget directly
    // if the layout has children that have constraints outside of the subtree, those
    // constraints will not be removed. todo: find an efficient way of resolving this
    pub fn hide_widget(&mut self, id: LayoutId) {
        if !self.layouts.hidden_layouts.contains_key(&id) {
            let mut constraints = Vec::new();
            let layout = &self.layouts.layouts[&id];
            for var in layout.array().iter() {
                if let Some(constraint_set) = self.layouts.var_constraints.get(&var) {
                    for constraint in constraint_set {
                        if self.solver.has_constraint(&constraint) {
                            self.solver.remove_constraint(&constraint).unwrap();
                        }
                        constraints.push(constraint.clone());
                    }
                }
            }
            self.layouts.hidden_layouts.insert(id, constraints);
        }
    }
    pub fn unhide_widget(&mut self, id: LayoutId) {
        if let Some(constraints) = self.layouts.hidden_layouts.remove(&id) {
            for constraint in constraints {
                if !self.solver.has_constraint(&constraint) {
                    self.solver.add_constraint(constraint).unwrap();
                }
            }
        }
    }
    pub fn update_solver<F>(&mut self, f: F)
        where F: Fn(&mut cassowary::Solver)
    {
        f(&mut self.solver);
    }

    pub fn has_edit_variable(&mut self, v: &Variable) -> bool {
        self.solver.has_edit_variable(v)
    }
    pub fn has_constraint(&self, constraint: &Constraint) -> bool {
        self.solver.has_constraint(constraint)
    }

    pub fn edit_variable(&mut self, var: Variable, val: f64) {
        if !self.solver.has_edit_variable(&var) {
            let strength = self.layouts.edit_strengths.remove(&var).unwrap_or(strength::STRONG);
            self.solver.add_edit_variable(var, strength).unwrap();
        }
        self.suggest_value(var, val);
    }

    fn suggest_value(&mut self, var: Variable, val: f64) {
        if val.is_finite() {
            self.solver.suggest_value(var, val).unwrap();
            debug!("suggest edit_var {} {}", self.layouts.fmt_variable(var), val);
        } else {
            debug!("invalid edit_var {} {}", self.layouts.fmt_variable(var), val);
        }
    }

    pub fn update_layout(&mut self, layout: &mut Layout) {
        self.layouts.layout_names.insert(layout.id, layout.name.clone());
        for edit_var in layout.get_edit_vars() {
            if let Some(val) = edit_var.val {
                if !self.solver.has_edit_variable(&edit_var.var) {
                    debug!("add edit_var {}", self.layouts.fmt_variable(edit_var.var));
                    self.solver.add_edit_variable(edit_var.var, edit_var.strength).unwrap();
                }
                if val.is_finite() {
                    self.solver.suggest_value(edit_var.var, val).unwrap();
                    debug!("suggest edit_var {} {}", self.layouts.fmt_variable(edit_var.var), val);
                } else {
                    debug!("invalid edit_var {} {}", self.layouts.fmt_variable(edit_var.var), val);
                }
            } else {
                self.layouts.edit_strengths.insert(edit_var.var, edit_var.strength);
            }
        }
        for constraint in layout.get_removed_constraints() {
            if self.solver.has_constraint(&constraint) {
                self.solver.remove_constraint(&constraint).unwrap();
            }
        }
        for constraint in layout.get_constraints() {
            if self.solver.add_constraint(constraint.clone()).is_err() {
                eprintln!("Failed to add constraint {}", self.layouts.fmt_constraint(&constraint));
                self.debug_constraints();
            }
            let var_list = self.layouts.constraint_vars.entry(constraint.clone()).or_insert(Vec::new());
            for term in &constraint.0.expression.terms {
                let variable = term.variable;
                let constraint_set = self.layouts.var_constraints.entry(variable).or_insert(HashSet::new());
                constraint_set.insert(constraint.clone());
                var_list.push(variable);
            }
        }
    }

    pub fn fetch_changes(&mut self) -> Vec<(LayoutId, VarType, f64)> {
        let mut changes = Vec::new();
        for &(var, val) in self.solver.fetch_changes() {
            debug!("solver {} = {}", self.layouts.fmt_variable(var), val);
            if let Some(widget_id) = self.layouts.var_ids.get(&var) {
                let var_type = self.layouts.var_types[&var];
                changes.push((*widget_id, var_type, val));
            }
        }
        changes
    }

    pub fn debug_variables(&self) {
        println!("VARIABLES");
        for var in self.layouts.var_constraints.keys() {
            println!("{}", self.layouts.fmt_variable(*var));
        }
    }

    pub fn debug_constraints(&self) {
        println!("CONSTRAINTS");
        for constraint in self.layouts.constraint_vars.keys() {
            self.debug_constraint(constraint);
        }
    }

    pub fn debug_constraint(&self, constraint: &Constraint) {
        println!("{}", self.layouts.fmt_constraint(constraint));
    }
}

#[derive(Debug, Clone, Copy)]
pub enum VarType {
    Left,
    Top,
    Right,
    Bottom,
    Width,
    Height,
}

pub struct LayoutManager {
    var_constraints: HashMap<Variable, HashSet<Constraint>>,
    constraint_vars: HashMap<Constraint, Vec<Variable>>,
    var_ids: HashMap<Variable, LayoutId>,
    var_types: HashMap<Variable, VarType>,
    pub layouts: HashMap<LayoutId, LayoutVars>,
    layout_names: HashMap<LayoutId, Option<String>>,
    last_layout: LayoutId,
    hidden_layouts: HashMap<LayoutId, Vec<Constraint>>,
    edit_strengths: HashMap<Variable, f64>,
}

impl LayoutManager {

    pub fn new() -> Self {
        LayoutManager {
            var_constraints: HashMap::new(),
            constraint_vars: HashMap::new(),
            var_ids: HashMap::new(),
            var_types: HashMap::new(),
            layouts: HashMap::new(),
            layout_names: HashMap::new(),
            last_layout: 0,
            hidden_layouts: HashMap::new(),
            edit_strengths: HashMap::new(),
        }
    }
    pub fn register_widget(&mut self, layout: &mut Layout) {
        let id = layout.id;
        if id > self.last_layout {
            self.last_layout = id;
        }
        self.var_ids.insert(layout.vars.left, id);
        self.var_ids.insert(layout.vars.top, id);
        self.var_ids.insert(layout.vars.right, id);
        self.var_ids.insert(layout.vars.bottom, id);
        self.var_ids.insert(layout.vars.width, id);
        self.var_ids.insert(layout.vars.height, id);

        self.var_types.insert(layout.vars.left, VarType::Left);
        self.var_types.insert(layout.vars.top, VarType::Top);
        self.var_types.insert(layout.vars.right, VarType::Right);
        self.var_types.insert(layout.vars.bottom, VarType::Bottom);
        self.var_types.insert(layout.vars.width, VarType::Width);
        self.var_types.insert(layout.vars.height, VarType::Height);

        self.layouts.insert(id, layout.vars.clone());
        self.layout_names.insert(id, layout.name.clone());
    }

    pub fn fmt_variable(&self, var: Variable) -> String {
        let layout = &self.var_ids[&var];
        let var_type = &self.var_types[&var];
        let layout_name = &self.layout_names[layout];
        format!("{:?}.{:?}", layout_name, var_type)
    }

    pub fn fmt_constraint(&self, constraint: &Constraint) -> String {
        let ref constraint = constraint.0;
        let strength_desc = {
            let stren = constraint.strength;
            if stren < strength::WEAK { "WEAK-" }
            else if stren == strength::WEAK { "WEAK " }
            else if stren < strength::MEDIUM { "WEAK+" }
            else if stren == strength::MEDIUM { "MED  " }
            else if stren < strength::STRONG { "MED+ " }
            else if stren == strength::STRONG { "STR  " }
            else if stren < strength::REQUIRED { "STR+ " }
            else if stren == strength::REQUIRED { "REQD " }
            else { "REQD+" }
        };
        format!("{} {} {} 0", strength_desc, self.fmt_expression(&constraint.expression), constraint.op)
    }
    fn fmt_expression(&self, expression: &Expression) -> String {
        let mut out = String::new();
        let mut first = true;
        if expression.constant != 0.0 {
            write!(out, "{}", expression.constant).unwrap();
            first = false;
        }
        for term in expression.terms.iter() {
            let coef = {
                if term.coefficient == 1.0 {
                    if first {
                        "".to_owned()
                    } else {
                        "+ ".to_owned()
                    }
                } else if term.coefficient == -1.0 {
                    "- ".to_owned()
                } else if term.coefficient > 0.0 {
                    if !first {
                        format!("+ {} * ", term.coefficient)
                    } else {
                        format!("{} * ", term.coefficient)
                    }
                } else {
                    format!("- {} * ", term.coefficient)
                }
            };
            write!(out, " {}{}", coef, self.fmt_variable(term.variable)).unwrap();

            first = false;
        }
        out
    }
}
