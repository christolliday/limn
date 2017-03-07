use std::collections::{HashMap, HashSet};

use cassowary;
use cassowary::{Variable, Constraint};

use resources::WidgetId;
use widget::Widget;
use widget::layout::WidgetConstraint;
use event::{Target, Queue};
use ui::LayoutChanged;

/// wrapper around cassowary solver that keeps widgets positions in sync, sends events when layout changes happen
pub struct LimnSolver {
    solver: cassowary::Solver,
    var_map: HashMap<Variable, WidgetId>,
    constraint_map: HashMap<WidgetId, Vec<Constraint>>,
    queue: Queue,
}

impl LimnSolver {
    pub fn new(queue: Queue) -> Self {
        LimnSolver {
            solver: cassowary::Solver::new(),
            var_map: HashMap::new(),
            constraint_map: HashMap::new(),
            queue: queue,
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
            self.solver.add_constraint(constraint).unwrap();
        }

        let ref vars = widget.layout;
        self.var_map.insert(vars.left, widget.id);
        self.var_map.insert(vars.top, widget.id);
        self.var_map.insert(vars.right, widget.id);
        self.var_map.insert(vars.bottom, widget.id);
        self.check_changes();
    }
    pub fn remove_widget(&mut self, widget_id: &WidgetId) {
        // remove constraints that are relative to this widget from solver
        if let Some(constraint_list) = self.constraint_map.get(&widget_id) {
            for constraint in constraint_list {
                if self.solver.has_constraint(constraint) {
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

    pub fn fetch_changes(&mut self) -> &[(Variable, f64)] {
        self.solver.fetch_changes()
    }
    pub fn get_value(&mut self, v: Variable) -> f64 {
        self.solver.get_value(v)
    }

    fn check_changes(&mut self) {
        let changes = self.solver.fetch_changes();
        if changes.len() > 0 {
            let mut widget_ids = HashSet::new();
            for &(var, _) in changes {
                if let Some(widget_id) = self.var_map.get(&var) {
                    widget_ids.insert(widget_id.clone());
                }
            }
            for widget_id in widget_ids {
                self.queue.push(Target::Ui, LayoutChanged(widget_id));
            }
        }
    }
}
