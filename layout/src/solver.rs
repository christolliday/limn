use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Write;

use cassowary;
use cassowary::strength;
use cassowary::strength::*;
use cassowary::{Variable, Constraint, Expression};
use cassowary::WeightedRelation::*;

use super::{LayoutId, Layout, VarType, LayoutVars, EditVariable, Rect, Point, Size};

pub struct LimnSolver {
    pub solver: cassowary::Solver,
    pub strict: bool,
    layouts: LayoutManager,
}

impl LimnSolver {
    pub fn new() -> Self {
        LimnSolver {
            solver: cassowary::Solver::new(),
            strict: false,
            layouts: LayoutManager::new(),
        }
    }

    pub fn update_layout(&mut self, layout: &mut Layout) {

        let registered = self.layouts.layouts.contains_key(&layout.id);
        if !registered {
            self.layouts.register_layout(layout);
        }
        self.layouts.update_layout(layout);
        for constraint in layout.get_removed_constraints() {
            self.layouts.remove_constraint(&constraint);
            self.remove_constraint(&constraint);
        }
        if !registered {
            for constraint in self.layouts.dequeue_constraints(layout) {
                self.add_constraint(constraint.clone());
            }
        }

        if layout.hidden && !self.layouts.layout_hidden(layout.id) {
            self.hide_layout(layout.id);
        } else if !layout.hidden && self.layouts.layout_hidden(layout.id) {
            self.unhide_layout(layout.id);
        }
        for constraint in layout.get_constraints() {
            if self.layouts.add_constraint(&constraint) {
                self.add_constraint(constraint.clone());
            }
        }
        for edit_var in layout.get_edit_vars() {
            self.update_edit_var(&edit_var);
            self.layouts.update_edit_var(layout.id, edit_var);
        }
    }
    fn update_edit_var(&mut self, edit_var: &EditVariable) {
        let &EditVariable { var, val, strength } = edit_var;
        if !self.solver.has_edit_variable(&var) {
            debug!("add edit_var {}", self.layouts.fmt_variable(var));
            self.solver.add_edit_variable(var, strength).unwrap();
        }
        if val.is_finite() {
            self.solver.suggest_value(var, val).unwrap();
            debug!("suggest edit_var {} {}", self.layouts.fmt_variable(var), val);
        } else {
            debug!("invalid edit_var {} {}", self.layouts.fmt_variable(var), val);
        }
    }
    fn add_constraint(&mut self, constraint: Constraint) {
        debug!("adding constraint {}", self.layouts.fmt_constraint(&constraint));
        if self.solver.add_constraint(constraint.clone()).is_err() {
            eprintln!("Failed to add constraint {}", self.layouts.fmt_constraint(&constraint));
            self.debug_associated_constraints(&constraint);
            if self.strict {
                panic!("Solver unsatisfiable");
            }
        }
    }

    fn remove_constraint(&mut self, constraint: &Constraint) {
        debug!("removing constraint {}", self.layouts.fmt_constraint(constraint));
        if self.solver.has_constraint(constraint) {
            self.solver.remove_constraint(constraint).unwrap();
        }
    }

    pub fn remove_layout(&mut self, id: LayoutId) {
        if let Some(layout) = self.layouts.layouts.remove(&id) {
            for constraint in layout.constraints {
                self.remove_constraint(&constraint);
            }
            for var in layout.vars.array().iter() {
                self.layouts.var_ids.remove(&var);
            }
        }
    }

    pub fn hide_layout(&mut self, id: LayoutId) {
        if !self.layouts.layout_hidden(id) {
            for constraint in self.layouts.layouts[&id].constraints.clone() {
                self.remove_constraint(&constraint);
            }
            {
                let layout = self.layouts.layouts.get_mut(&id).unwrap();
                if layout.hidden_constraints.len() == 0 {
                    layout.hidden_constraints.extend(vec![
                        layout.vars.width | EQ(REQUIRED) | 0.0, layout.vars.height | EQ(REQUIRED) | 0.0,
                        layout.vars.left | EQ(REQUIRED) | 0.0, layout.vars.top | EQ(REQUIRED) | 0.0,
                        layout.vars.right | EQ(REQUIRED) | 0.0, layout.vars.bottom | EQ(REQUIRED) | 0.0,
                    ]);
                }
                layout.hidden = true;
            }
            for constraint in self.layouts.layouts[&id].hidden_constraints.clone() {
                self.add_constraint(constraint.clone());
            }
        }
        let children = self.layouts.layouts[&id].children.clone();
        for child in children {
            self.hide_layout(child);
        }
    }
    pub fn unhide_layout(&mut self, id: LayoutId) {
        if self.layouts.layout_hidden(id) {
            for constraint in self.layouts.layouts[&id].hidden_constraints.clone() {
                self.remove_constraint(&constraint);
            }
            for constraint in self.layouts.layouts[&id].constraints.clone() {
                if !self.solver.has_constraint(&constraint) {
                    let mut hidden = false;
                    for layout_id in self.layouts.dependent_layouts(&constraint) {
                        if layout_id != id && self.layouts.layout_hidden(layout_id) {
                            hidden = true;
                            break;
                        }
                    }
                    if !hidden {
                        self.add_constraint(constraint.clone());
                    }
                }
            }
            let layout = self.layouts.layouts.get_mut(&id).unwrap();
            layout.hidden = false;
        }
        let children = self.layouts.layouts[&id].children.clone();
        for child in children {
            self.unhide_layout(child);
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

    pub fn fetch_changes(&mut self) -> Vec<(LayoutId, VarType, f64)> {
        let mut changes = Vec::new();
        for &(var, val) in self.solver.fetch_changes() {
            debug!("solver {} = {}", self.layouts.fmt_variable(var), val);
            if let Some(layout_id) = self.layouts.var_ids.get(&var) {
                let var_type = self.layouts.layouts[&layout_id].vars.var_type(var);
                changes.push((*layout_id, var_type, val));
            }
        }
        changes
    }

    pub fn debug_variables(&self) {
        println!("VARIABLES");
        for var in self.layouts.var_ids.keys() {
            println!("{}", self.layouts.fmt_variable(*var));
        }
    }

    pub fn debug_constraints(&self) {
        println!("CONSTRAINTS");
        let mut shown_constraints = HashSet::new();
        let mut layouts = VecDeque::new();
        layouts.push_front(self.layouts.root);
        while let Some(layout) = layouts.pop_front() {
            println!("{}", self.layouts.layout_name(layout).to_uppercase());
            for constraint in &self.layouts.layouts[&layout].constraints {
                if !shown_constraints.contains(constraint) {
                    self.debug_constraint(constraint);
                    shown_constraints.insert(constraint.clone());
                }
            }
            println!("EDIT_VARS");
            for (_, edit_var) in &self.layouts.layouts[&layout].edit_vars {
                println!("{}", self.layouts.fmt_edit_variable(edit_var));
            }
            layouts.extend(self.layouts.children(layout));
        }
    }

    pub fn debug_constraint(&self, constraint: &Constraint) {
        println!("{}", self.layouts.fmt_constraint(constraint));
    }

    pub fn debug_associated_constraints(&self, constraint: &Constraint) {
        let mut visited_constraints = HashSet::new();
        let mut new_constraints = HashSet::new();
        new_constraints.insert(constraint.clone());

        loop {
            if new_constraints.len() == 0 {
                break;
            }
            let mut newer_constraints = HashSet::new();
            for constraint in new_constraints.drain() {
                for var in constraint_vars(&constraint) {
                    for constraint in self.layouts.constraints_for(var) {
                        if !visited_constraints.contains(constraint) {
                            newer_constraints.insert(constraint.clone());
                        }
                    }
                }
                visited_constraints.insert(constraint);
            }
            new_constraints = newer_constraints;
        }
        for constraint in visited_constraints {
            self.debug_constraint(&constraint);
        }
    }

    pub fn debug_layouts(&self) {
        println!("LAYOUTS");
        let mut layouts = VecDeque::new();
        layouts.push_front(self.layouts.root);
        while let Some(layout) = layouts.pop_front() {
            self.debug_layout(layout);
            layouts.extend(self.layouts.children(layout));
        }
    }

    pub fn debug_layout(&self, id: LayoutId) {
        let bounds = {
            let get_val = |var| self.solver.get_value(var) as f32;
            let vars = &self.layouts.layouts[&id].vars;
            let origin = Point::new(get_val(vars.left), get_val(vars.top));
            let size = Size::new(get_val(vars.width), get_val(vars.height));
            Rect::new(origin, size)
        };
        println!("{} {}", self.layouts.layout_name(id), bounds);
    }
}

fn constraint_vars(constraint: &Constraint) -> Vec<Variable> {
    constraint.expr().terms.iter().map(|term| term.variable).collect()
}

struct LayoutInternal {
    vars: LayoutVars,
    name: Option<String>,
    associated_vars: HashMap<Variable, String>,
    edit_vars: HashMap<Variable, EditVariable>,
    constraints: HashSet<Constraint>,
    children: Vec<LayoutId>,
    hidden: bool,
    hidden_constraints: Vec<Constraint>,
}
pub struct LayoutManager {
    root: LayoutId,
    var_ids: HashMap<Variable, LayoutId>,
    layouts: HashMap<LayoutId, LayoutInternal>,
    constraints: HashMap<Variable, HashSet<Constraint>>,

    pending_constraints: HashMap<Variable, Vec<Constraint>>,
    missing_vars: HashMap<Constraint, usize>,
}

impl LayoutManager {

    pub fn new() -> Self {
        LayoutManager {
            root: 0,
            var_ids: HashMap::new(),
            layouts: HashMap::new(),
            constraints: HashMap::new(),
            pending_constraints: HashMap::new(),
            missing_vars: HashMap::new(),
        }
    }

    pub fn register_layout(&mut self, layout: &mut Layout) {
        let id = layout.id;

        for var in layout.vars.array().iter() {
            self.var_ids.insert(*var, id);
        }
        let layout = LayoutInternal {
            vars: layout.vars.clone(),
            name: layout.name.clone(),
            associated_vars: HashMap::new(),
            edit_vars: HashMap::new(),
            constraints: HashSet::new(),
            children: layout.children.clone(),
            hidden: false,
            hidden_constraints: Vec::new(),
        };
        self.layouts.insert(id, layout);
    }

    pub fn update_layout(&mut self, layout: &mut Layout) {
        let internal_layout = self.layouts.get_mut(&layout.id).unwrap();
        for (var, name) in layout.get_associated_vars() {
            self.var_ids.insert(var, layout.id);
            internal_layout.associated_vars.insert(var, name);
        }
        internal_layout.name = layout.name.clone();
    }

    pub fn add_constraint(&mut self, constraint: &Constraint) -> bool {
        let mut missing_layouts = false;
        for term in &constraint.expr().terms {
            self.constraints.entry(term.variable).or_insert_with(HashSet::new).insert(constraint.clone());
            if self.var_ids.contains_key(&term.variable) {
                let layout_id = self.var_ids[&term.variable];
                self.layouts.get_mut(&layout_id).unwrap().constraints.insert(constraint.clone());
            } else {
                missing_layouts = true;
                self.queue_constraint(term.variable, constraint.clone());
            }
        }
        !missing_layouts
    }

    pub fn remove_constraint(&mut self, constraint: &Constraint) {
        for term in &constraint.expr().terms {
            self.constraints.entry(term.variable).or_insert_with(HashSet::new).remove(constraint);
            if self.var_ids.contains_key(&term.variable) {
                let layout_id = self.var_ids[&term.variable];
                self.layouts.get_mut(&layout_id).unwrap().constraints.remove(constraint);
            }
        }
    }
    fn update_edit_var(&mut self, layout_id: LayoutId, edit_var: EditVariable) {
        self.layouts.get_mut(&layout_id).unwrap().edit_vars.insert(edit_var.var, edit_var);
    }

    fn constraints_for(&self, variable: Variable) -> &HashSet<Constraint> {
        &self.constraints[&variable]
    }

    // if constraint added for non-registered variable, add to pending and increment counter
    // of missing variables for that constraint
    pub fn queue_constraint(&mut self, variable: Variable, constraint: Constraint) {
        *self.missing_vars.entry(constraint.clone()).or_insert(0) += 1;
        self.pending_constraints.entry(variable).or_insert_with(Vec::new)
            .push(constraint);
    }
    // when new layout registered, if there are any pending constraints for it's variables
    // and no other variables are missing for them, add those constraints
    pub fn dequeue_constraints(&mut self, layout: &mut Layout) -> Vec<Constraint> {
        let constraints = {
            let mut constraints = Vec::new();
            for var in self.layout_vars(layout.id) {
                if let Some(pending) = self.pending_constraints.remove(&var) {
                    for constraint in pending {
                        *self.missing_vars.get_mut(&constraint).unwrap() -= 1;
                        if self.missing_vars[&constraint] == 0 {
                            self.missing_vars.remove(&constraint);
                            constraints.push(constraint);
                        }
                    }
                }
            }
            constraints
        };
        for constraint in &constraints {
            for term in &constraint.expr().terms {
                let layout_id = self.var_ids[&term.variable];
                self.layouts.get_mut(&layout_id).unwrap().constraints.insert(constraint.clone());
            }
        }
        constraints
    }

    pub fn layout_hidden(&self, id: LayoutId) -> bool {
        self.layouts[&id].hidden
    }

    pub fn layout_name(&self, id: LayoutId) -> String {
        self.layouts[&id].name.clone().unwrap_or("unknown".to_owned())
    }

    fn dependent_layouts(&self, constraint: &Constraint) -> Vec<LayoutId> {
        constraint.expr().terms.iter().map(|term| self.var_ids[&term.variable]).collect()
    }

    pub fn children(&self, id: LayoutId) -> Vec<LayoutId> {
        self.layouts[&id].children.clone()
    }
    pub fn layout_vars(&self, id: LayoutId) -> Vec<Variable> {
        let layout = &self.layouts[&id];
        layout.vars.array().iter().chain(layout.associated_vars.keys()).map(|var| *var).collect()
    }

    pub fn fmt_variable(&self, var: Variable) -> String {
        let id = self.var_ids[&var];
        let layout = &self.layouts[&id];
        let layout_name = layout.name.clone().unwrap_or("unknown".to_owned());
        let var_type = layout.vars.var_type(var);
        let var_type = if let VarType::Other = var_type {
            layout.associated_vars[&var].to_owned()
        } else {
            format!("{:?}", var_type).to_lowercase()
        };
        format!("{}.{}", layout_name, var_type)
    }

    pub fn fmt_edit_variable(&self, edit_var: &EditVariable) -> String {
        format!("{} {} == {}", strength_desc(edit_var.strength), self.fmt_variable(edit_var.var), edit_var.val)
    }

    pub fn fmt_constraint(&self, constraint: &Constraint) -> String {
        format!("{} {}", strength_desc(constraint.strength()), self.fmt_expression(&constraint.expr(), constraint.op()))
    }

    fn fmt_expression(&self, expression: &Expression, op: cassowary::RelationalOperator) -> String {
        let (mut neg_terms, mut pos_terms) = (Vec::new(), Vec::new());
        for term in expression.terms.iter() {
            let neg = term.coefficient < 0.0;
            let coef = term.coefficient.abs();
            let coef = if coef == 1.0 { "".to_owned() } else { coef.to_string() };
            let desc = format!("{}{}", coef, self.fmt_variable(term.variable));
            if neg { neg_terms.push(desc) } else { pos_terms.push(desc) }
        }
        if expression.constant != 0.0 {
            let neg = expression.constant < 0.0;
            let desc = expression.constant.abs().to_string();
            if neg { neg_terms.push(desc) } else { pos_terms.push(desc) }
        }
        let mut out = String::new();
        if pos_terms.len() == 0 {
            write!(out, "0").unwrap();
        } else {
            let mut terms = pos_terms.iter();
            let term = terms.next().unwrap();
            write!(out, "{}", term).unwrap();
            for term in terms {
                write!(out, " + {}", term).unwrap();
            }
        }
        write!(out, " {} ", op).unwrap();
        if neg_terms.len() == 0 {
            write!(out, "0").unwrap();
        } else {
            let mut terms = neg_terms.iter();
            let term = terms.next().unwrap();
            write!(out, "{}", term).unwrap();
            for term in terms {
                write!(out, " + {}", term).unwrap();
            }
        }
        out
    }
}

fn strength_desc(strength: f64) -> &'static str {
    if strength < strength::WEAK { "WEAK-" }
    else if strength == strength::WEAK { "WEAK " }
    else if strength < strength::MEDIUM { "WEAK+" }
    else if strength == strength::MEDIUM { "MED  " }
    else if strength < strength::STRONG { "MED+ " }
    else if strength == strength::STRONG { "STR  " }
    else if strength < strength::REQUIRED { "STR+ " }
    else if strength == strength::REQUIRED { "REQD " }
    else { "REQD+" }
}
