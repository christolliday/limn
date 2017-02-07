use std::collections::{HashMap, HashSet};

use cassowary;
use cassowary::{Variable, Constraint};
use cassowary::{AddEditVariableError, RemoveEditVariableError, SuggestValueError};
use cassowary::{AddConstraintError, RemoveConstraintError};

use petgraph::graph::NodeIndex;

use resources::WidgetId;
use widget::Widget;
use widget::layout::LayoutVars;
use event::{EventAddress, EventQueue};
use event::id::*;

/// wrapper around cassowary solver that keeps widgets positions in sync, sends events when layout changes happen
pub struct LimnSolver {
    solver: cassowary::Solver,
    var_map: HashMap<Variable, WidgetId>,
    event_queue: EventQueue,
}

impl LimnSolver {
    pub fn new(event_queue: EventQueue) -> Self {
        LimnSolver {
            solver: cassowary::Solver::new(),
            var_map: HashMap::new(),
            event_queue: event_queue,
        }
    }
    pub fn add_widget(&mut self, widget: &Widget) {
        let ref vars = widget.layout;
        self.var_map.insert(vars.left, widget.id);
        self.var_map.insert(vars.top, widget.id);
        self.var_map.insert(vars.right, widget.id);
        self.var_map.insert(vars.bottom, widget.id);
    }

    pub fn add_edit_variable(&mut self, v: Variable, strength: f64) -> Result<(), AddEditVariableError> {
        let res = self.solver.add_edit_variable(v, strength);
        self.check_changes();
        res
    }
    pub fn remove_edit_variable(&mut self, v: Variable) -> Result<(), RemoveEditVariableError> {
        let res = self.solver.remove_edit_variable(v);
        self.check_changes();
        res
    }
    pub fn has_edit_variable(&mut self, v: &Variable) -> bool {
        self.solver.has_edit_variable(v)
    }
    pub fn suggest_value(&mut self, variable: Variable, value: f64) -> Result<(), SuggestValueError> {
        let res = self.solver.suggest_value(variable, value);
        self.check_changes();
        res
    }

    pub fn add_constraint(&mut self, constraint: Constraint) -> Result<(), AddConstraintError> {
        let res = self.solver.add_constraint(constraint);
        self.check_changes();
        res
    }
    pub fn remove_constraint(&mut self, constraint: &Constraint) -> Result<(), RemoveConstraintError> {
        let res = self.solver.remove_constraint(constraint);
        self.check_changes();
        res
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
            for &(var, val) in changes {
                if let Some(widget_id) = self.var_map.get(&var) {
                    widget_ids.insert(widget_id.clone());
                }
            }
            for widget_id in widget_ids {
                self.event_queue.push(EventAddress::Ui, LAYOUT, Box::new(widget_id));
            }
        }
    }
}
