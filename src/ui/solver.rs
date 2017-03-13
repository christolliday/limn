use std;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use linked_hash_map::LinkedHashMap;

use cassowary;
use cassowary::strength;
use cassowary::{Variable, Constraint};

use resources::WidgetId;
use widget::Widget;
use widget::layout::WidgetConstraint;
use event::{Target, Queue};
use ui::RedrawEvent;
use ui;

/// wrapper around cassowary solver that keeps widgets positions in sync, sends events when layout changes happen
pub struct LimnSolver {
    solver: cassowary::Solver,
    var_map: HashMap<Variable, WidgetId>,
    constraint_map: HashMap<WidgetId, Vec<Constraint>>,
    queue: Queue,
    debug_constraint_list: LinkedHashMap<Constraint, ()>, // LinkedHashSet (maintains insertion order)
}

impl LimnSolver {
    pub fn new(queue: Queue) -> Self {
        LimnSolver {
            solver: cassowary::Solver::new(),
            var_map: HashMap::new(),
            constraint_map: HashMap::new(),
            queue: queue,
            debug_constraint_list: LinkedHashMap::new(),
        }
    }
    pub fn add_widget(&mut self, widget: &Widget, constraints: Vec<WidgetConstraint>) {
        self.constraint_map.insert(widget.id, Vec::new());
        for constraint in constraints {
            // insert constraint into list for both widgets it affects,
            // so that if either widget is removed, the constraint is as well
            let constraint = match constraint {
                WidgetConstraint::Local(constraint) => constraint,
                WidgetConstraint::Relative(constraint, widget_id) => {
                    if !self.constraint_map.contains_key(&widget_id) {
                        self.constraint_map.insert(widget_id, Vec::new());
                    }
                    if let Some(constraint_list) = self.constraint_map.get_mut(&widget.id) {
                        constraint_list.push(constraint.clone());
                    }
                    constraint
                }
            };
            if let Some(constraint_list) = self.constraint_map.get_mut(&widget.id) {
                constraint_list.push(constraint.clone());
            }
            self.add_constraint(constraint);
        }

        let ref vars = widget.layout;
        self.var_map.insert(vars.left, widget.id);
        self.var_map.insert(vars.top, widget.id);
        self.var_map.insert(vars.right, widget.id);
        self.var_map.insert(vars.bottom, widget.id);
        self.check_changes();

        if let Some(ref debug_name) = widget.debug_name {
            add_debug_var_name(widget.layout.left, &format!("{}.left", debug_name));
            add_debug_var_name(widget.layout.top, &format!("{}.top", debug_name));
            add_debug_var_name(widget.layout.right, &format!("{}.right", debug_name));
            add_debug_var_name(widget.layout.bottom, &format!("{}.bottom", debug_name));
        }
    }
    pub fn remove_widget(&mut self, widget_id: &WidgetId) {
        // remove constraints that are relative to this widget from solver
        if let Some(constraint_list) = self.constraint_map.get(&widget_id) {
            for constraint in constraint_list {
                if self.solver.has_constraint(constraint) {
                    self.debug_constraint_list.remove(constraint);
                    self.solver.remove_constraint(constraint).unwrap();
                }
            }
        }
        // doesn't clean up other references to these constraints in the constraint map, but at least they won't affect the solver
        self.constraint_map.remove(&widget_id);
        self.check_changes();
    }
    pub fn update_solver<F>(&mut self, f: F)
        where F: Fn(&mut cassowary::Solver)
    {
        f(&mut self.solver);
        self.check_changes();
    }

    pub fn has_edit_variable(&mut self, v: &Variable) -> bool {
        self.solver.has_edit_variable(v)
    }
    pub fn has_constraint(&self, constraint: &Constraint) -> bool {
        self.solver.has_constraint(constraint)
    }

    fn add_constraint(&mut self, constraint: Constraint) {
        self.debug_constraint_list.insert(constraint.clone(), ());
        self.solver.add_constraint(constraint).unwrap();
    }

    fn check_changes(&mut self) {
        //self.debug_constraints();
        let changes = self.solver.fetch_changes();
        if changes.len() > 0 {
            let mut wchanges = Vec::new();
            for &(var, que) in changes {
                if let Some(widget_id) = self.var_map.get(&var) {
                    wchanges.push((*widget_id, var, que));
                }
            }
            self.queue.push(Target::Ui, LayoutChanged(wchanges));
        }
    }
    pub fn debug_constraints(&self) {
        println!("CONSTRAINTS");
        for constraint in self.debug_constraint_list.keys() {
            debug_constraint(constraint);
        }
    }
}

pub struct LayoutChanged(Vec<(WidgetId, Variable, f64)>);
pub struct LayoutChangeHandler;
impl ui::EventHandler<LayoutChanged> for LayoutChangeHandler {
    fn handle(&mut self, event: &LayoutChanged, args: ui::EventArgs) { 
        let ref changes = event.0;
        for &(widget_id, var, value) in changes {
            if let Some(widget) = args.ui.graph.get_widget(widget_id) {
                widget.layout.update_val(var, value);
            }
        }
        // redraw everything when layout changes, for now
        args.queue.push(Target::Ui, RedrawEvent);
        args.ui.graph.redraw();
    }
}

// Below code is used to debug cassowary constraints
// uses unsafe to access private fields, so depends on matching cassowary struct contents exactly
// This functionality could be added to cassowary at some point, but the design will need more thought

fn debug_constraint(constraint: &Constraint) {
    let constraint = unsafe {
        std::mem::transmute::<&Constraint, &DebugConstraint>(constraint)
    };
    println!("{:?}", constraint);
}
struct DebugConstraint(Arc<DebugConstraintData>);
struct DebugConstraintData {
    expression: DebugExpression,
    strength: f64,
    op: DebugRelationalOperator
}
#[allow(dead_code)]
enum DebugRelationalOperator {
    LessOrEqual,
    Equal,
    GreaterOrEqual
}
pub struct DebugExpression {
    pub terms: Vec<DebugTerm>,
    pub constant: f64
}
pub struct DebugTerm {
    pub variable: DebugVariable,
    pub coefficient: f64
}
pub struct DebugVariable(usize);

impl std::fmt::Debug for DebugConstraint {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        let strength_desc = {
            let stren = self.0.strength;
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
        write!(fmt, "{} {:?} {} 0", strength_desc, self.0.expression, self.0.op)
    }
}
impl std::fmt::Display for DebugRelationalOperator {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            DebugRelationalOperator::LessOrEqual => try!(write!(fmt, "<=")),
            DebugRelationalOperator::Equal => try!(write!(fmt, "==")),
            DebugRelationalOperator::GreaterOrEqual => try!(write!(fmt, ">=")),
        }
        Ok(())
    }
}

impl std::fmt::Debug for DebugExpression {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut first = true;
        if self.constant != 0.0 {
            try!(write!(fmt, "{}", self.constant));
            first = false;
        }
        for term in self.terms.iter() {
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
            try!(write!(fmt, " {}{:?}", coef, term.variable));

            first = false;
        }
        Ok(())
    }
}
impl std::fmt::Debug for DebugVariable {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        let names = VAR_NAMES.lock().unwrap();
        let var = unsafe {
            std::mem::transmute::<&DebugVariable, &Variable>(self)
        };
        if let Some(name) = names.get(var) {
            write!(fmt, "{}", name)
        } else {
            write!(fmt, "var({})", self.0)
        }
    }
}

lazy_static! {
    pub static ref VAR_NAMES: Mutex<HashMap<Variable, String>> = Mutex::new(HashMap::new());
}
pub fn add_debug_var_name(var: Variable, name: &str) {
    let mut names = VAR_NAMES.lock().unwrap();
    names.insert(var, name.to_owned());
}