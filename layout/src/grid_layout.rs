use cassowary::strength::*;
use cassowary::WeightedRelation::*;

use super::{LayoutVars, Layout, Constraint};
use super::constraint::*;

pub struct GridLayout {
    num_columns: usize,
    column: usize,
    rows: Vec<LayoutVars>,
    columns: Vec<LayoutVars>,
    row_end: Option<Constraint>,
}

impl GridLayout {
    pub fn new(parent: &mut Layout, num_columns: usize) -> Self {
        assert!(num_columns > 0, "can't create grid layout with no columns");
        let mut columns = Vec::new();
        for col in 0..num_columns {
            let column = LayoutVars::new();
            let mut constraints = vec![
                column.top | EQ(REQUIRED) | parent.vars.top,
                column.bottom | EQ(REQUIRED) | parent.vars.bottom,
                column.right - column.left | EQ(REQUIRED) | column.width,
                column.bottom - column.top | EQ(REQUIRED) | column.height,
            ];
            if let Some(last_column) = columns.last() {
                let last_column: &LayoutVars = last_column;
                constraints.push(column.left | EQ(REQUIRED) | last_column.right);
                constraints.push(column.width | EQ(REQUIRED) | last_column.width);
            } else {
                constraints.push(column.left | EQ(REQUIRED) | parent.vars.left);
            }
            if col == num_columns - 1 {
                constraints.push(column.right | EQ(REQUIRED) | parent.vars.right);
            }
            parent.add(constraints);
            parent.add_associated_vars(&column, &format!("column_{}", col));
            columns.push(column);
        }
        GridLayout {
            num_columns: num_columns,
            column: 0,
            rows: Vec::new(),
            columns: columns,
            row_end: None,
        }
    }
    pub fn add_child_layout(&mut self, parent: &mut Layout, child: &mut Layout) {
        if self.column == self.num_columns || self.rows.len() == 0 {
            let row = LayoutVars::new();
            let mut constraints = vec![
                row.left | EQ(REQUIRED) | parent.vars.left,
                row.right | EQ(REQUIRED) | parent.vars.right,
                row.right - row.left | EQ(REQUIRED) | row.width,
                row.bottom - row.top | EQ(REQUIRED) | row.height,
            ];
            if let Some(last_row) = self.rows.last() {
                constraints.push(row.top | EQ(REQUIRED) | last_row.bottom);
                constraints.push(row.height | EQ(REQUIRED) | last_row.height);
            } else {
                constraints.push(row.top | EQ(REQUIRED) | parent.vars.top);
            }
            if let Some(row_end) = self.row_end.take() {
                parent.remove_constraint(row_end);
            }
            let row_end = row.bottom | EQ(REQUIRED) | parent.vars.bottom;
            self.row_end = Some(row_end.clone());
            constraints.push(row_end);
            parent.add(constraints);
            parent.add_associated_vars(&row, &format!("row_{}", self.rows.len()));
            self.rows.push(row);
            self.column = 0;
        }
        let (row, col) = (self.rows.last().unwrap(), self.columns.get(self.column).unwrap());
        child.add(constraints![
            bound_by(row),
            bound_by(col),
        ]);
        self.column += 1;
    }
}
