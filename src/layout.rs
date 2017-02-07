use cassowary;
use cassowary::{Variable, Constraint};
use cassowary::{AddEditVariableError, RemoveEditVariableError, SuggestValueError};
use cassowary::{AddConstraintError, RemoveConstraintError};

/// wrapper around cassowary solver that keeps widgets positions in sync, sends events when layout changes happen
pub struct LimnSolver {
    solver: cassowary::Solver,
}

impl LimnSolver {
    pub fn new() -> Self {
        LimnSolver { solver: cassowary::Solver::new() }
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
        // todo
    }
}
