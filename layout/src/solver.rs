use std::sync::Mutex;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;

use cassowary;
use cassowary::strength;
use cassowary::{Variable, Constraint, Expression};

use super::{LayoutId, Layout, Rect};

pub struct LimnSolver {
    pub solver: cassowary::Solver,
    var_constraints: HashMap<Variable, HashSet<Constraint>>,
    constraint_vars: HashMap<Constraint, Vec<Variable>>,
    var_ids: HashMap<Variable, LayoutId>,
    pub layouts: HashMap<LayoutId, Layout>,
    last_layout: LayoutId,
    hidden_layouts: HashMap<LayoutId, Vec<Constraint>>,
    edit_strengths: HashMap<Variable, f64>,
    missing_widget_layout: HashMap<Variable, f64>,
}

impl LimnSolver {
    pub fn new() -> Self {
        LimnSolver {
            solver: cassowary::Solver::new(),
            var_constraints: HashMap::new(),
            constraint_vars: HashMap::new(),
            var_ids: HashMap::new(),
            layouts: HashMap::new(),
            last_layout: 0,
            hidden_layouts: HashMap::new(),
            edit_strengths: HashMap::new(),
            missing_widget_layout: HashMap::new(),
        }
    }
    pub fn add_widget(&mut self, id: LayoutId, name: &Option<String>, layout: Layout, bounds: &mut Rect) {
        if id > self.last_layout {
            self.last_layout = id;
        }
        self.var_ids.insert(layout.vars.left, id);
        self.var_ids.insert(layout.vars.top, id);
        self.var_ids.insert(layout.vars.width, id);
        self.var_ids.insert(layout.vars.height, id);

        {
            let mut check_existing = |var| { self.missing_widget_layout.remove(var).unwrap_or(0.0) };
            bounds.origin.x = check_existing(&layout.vars.left);
            bounds.origin.y = check_existing(&layout.vars.top);
            bounds.size.width = check_existing(&layout.vars.width);
            bounds.size.height = check_existing(&layout.vars.height);
        }
        self.missing_widget_layout.remove(&layout.vars.right);
        self.missing_widget_layout.remove(&layout.vars.bottom);

        if let &Some(ref name) = name {
            add_debug_var_name(layout.vars.left, &format!("{}.left", name));
            add_debug_var_name(layout.vars.top, &format!("{}.top", name));
            add_debug_var_name(layout.vars.right, &format!("{}.right", name));
            add_debug_var_name(layout.vars.bottom, &format!("{}.bottom", name));
            add_debug_var_name(layout.vars.width, &format!("{}.width", name));
            add_debug_var_name(layout.vars.height, &format!("{}.height", name));
        }
        self.layouts.insert(id, layout);
        self.update_layout(id);
    }

    pub fn remove_widget(&mut self, id: LayoutId) {
        if let Some(layout) = self.layouts.remove(&id) {
            for var in layout.vars.array().iter() {
                // remove constraints that are relative to this widget from solver
                if let Some(constraint_set) = self.var_constraints.remove(&var) {
                    for constraint in constraint_set {
                        if self.solver.has_constraint(&constraint) {
                            self.solver.remove_constraint(&constraint).unwrap();
                            // look up other variables that references this constraint,
                            // and remove this constraint from those variables constraint sets
                            if let Some(var_list) = self.constraint_vars.get(&constraint) {
                                for var in var_list {
                                    if let Some(constraint_set) = self.var_constraints.get_mut(&var) {
                                        constraint_set.remove(&constraint);
                                    }
                                }
                            }
                        }
                    }
                }
                self.var_ids.remove(&var);
            }
        }
    }
    // hide/unhide are a simplified way of temporarily removing a layout, by removing
    // only the constraints on that widget directly
    // if the layout has children that have constraints outside of the subtree, those
    // constraints will not be removed. todo: find an efficient way of resolving this
    pub fn hide_widget(&mut self, id: LayoutId) {
        if !self.hidden_layouts.contains_key(&id) {
            let mut constraints = Vec::new();
            let layout = &self.layouts[&id];
            for var in layout.vars.array().iter() {
                if let Some(constraint_set) = self.var_constraints.get(&var) {
                    for constraint in constraint_set {
                        if self.solver.has_constraint(&constraint) {
                            self.solver.remove_constraint(&constraint).unwrap();
                        }
                        constraints.push(constraint.clone());
                    }
                }
            }
            self.hidden_layouts.insert(id, constraints);
        }
    }
    pub fn unhide_widget(&mut self, id: LayoutId) {
        if let Some(constraints) = self.hidden_layouts.remove(&id) {
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
            let strength = self.edit_strengths.remove(&var).unwrap_or(strength::STRONG);
            self.solver.add_edit_variable(var, strength).unwrap();
        }
        self.suggest_value(var, val);
    }

    fn suggest_value(&mut self, var: Variable, val: f64) {
        if val.is_finite() {
            self.solver.suggest_value(var, val).unwrap();
            debug!("suggest edit_var {} {}", fmt_variable(var), val);
        } else {
            debug!("invalid edit_var {} {}", fmt_variable(var), val);
        }
    }

    pub fn update_layout(&mut self, id: LayoutId) {
        let LimnSolver { ref mut layouts, ref mut solver, ref mut edit_strengths, ref mut constraint_vars, ref mut var_constraints, .. } = *self;
        let layout = layouts.get_mut(&id).unwrap();
        for edit_var in layout.get_edit_vars() {
            if let Some(val) = edit_var.val {
                if !solver.has_edit_variable(&edit_var.var) {
                    debug!("add edit_var {}", fmt_variable(edit_var.var));
                    solver.add_edit_variable(edit_var.var, edit_var.strength).unwrap();
                }
                if val.is_finite() {
                    solver.suggest_value(edit_var.var, val).unwrap();
                    debug!("suggest edit_var {} {}", fmt_variable(edit_var.var), val);
                } else {
                    debug!("invalid edit_var {} {}", fmt_variable(edit_var.var), val);
                }
            } else {
                edit_strengths.insert(edit_var.var, edit_var.strength);
            }
        }
        for constraint in layout.get_constraints() {
            solver.add_constraint(constraint.clone()).expect(&format!("Failed to add constraint {}", fmt_constraint(&constraint)));
            let var_list = constraint_vars.entry(constraint.clone()).or_insert(Vec::new());
            for term in &constraint.0.expression.terms {
                let variable = term.variable;
                let constraint_set = var_constraints.entry(variable).or_insert(HashSet::new());
                constraint_set.insert(constraint.clone());
                var_list.push(variable);
            }
        }
        for constraint in layout.get_removed_constraints() {
            if solver.has_constraint(&constraint) {
                solver.remove_constraint(&constraint).unwrap();
            }
        }
    }

    pub fn fetch_changes(&mut self) -> Vec<(LayoutId, Variable, f64)> {
        let mut changes = Vec::new();
        for &(var, val) in self.solver.fetch_changes() {
            debug!("solver {} = {}", fmt_variable(var), val);
            if let Some(widget_id) = self.var_ids.get(&var) {
                changes.push((*widget_id, var, val));
            } else {
                // widget doesn't exist in the widget map yet, because it hasn't been added yet
                // store it and use this to initialize the widget bounds when it is added
                self.missing_widget_layout.insert(var, val);
            }
        }
        changes
    }

    pub fn debug_constraints(&self) {
        println!("CONSTRAINTS");
        for i in 0..self.last_layout {
            if let Some(layout) = self.layouts.get(&i) {
                println!("{:?}", layout.name);
                for constraint in &layout.constraints {
                    debug_constraint(constraint);
                }
            }
        }
    }
    pub fn debug_variables(&mut self) {
        println!("VARIABLES");
        let names = VAR_NAMES.lock().unwrap();
        let mut vars: Vec<&Variable> = names.keys().collect();
        vars.sort();
        for variable in vars {
            let val = self.solver.get_value(*variable);
            if let Some(name) = names.get(&variable) {
                println!("{} = {}", name, val);
            } else {
                println!("var({:?}) = {}", variable, val);
            }
        }
    }
}

fn debug_constraint(constraint: &Constraint) {
    println!("{}", fmt_constraint(constraint));
}

pub fn fmt_constraint(constraint: &Constraint) -> String {
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
    format!("{} {} {} 0", strength_desc, fmt_expression(&constraint.expression), constraint.op)
}

fn fmt_expression(expression: &Expression) -> String {
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
        write!(out, " {}{}", coef, fmt_variable(term.variable)).unwrap();

        first = false;
    }
    out
}

pub fn fmt_variable(variable: Variable) -> String {
    let names = VAR_NAMES.lock().unwrap();
    if let Some(name) = names.get(&variable) {
        format!("{}", name)
    } else {
        format!("var({:?})", variable)
    }
}

lazy_static! {
    pub static ref VAR_NAMES: Mutex<HashMap<Variable, String>> = Mutex::new(HashMap::new());
}
pub fn add_debug_var_name(var: Variable, name: &str) {
    let mut names = VAR_NAMES.lock().unwrap();
    names.insert(var, name.to_owned());
}
