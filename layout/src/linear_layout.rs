use std::collections::HashMap;

use cassowary::strength::*;
use cassowary::WeightedRelation::*;
use cassowary::{Variable, Constraint};

use super::{LayoutId, LayoutVars, Layout};
use super::constraint::*;

#[derive(Copy, Clone)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Debug)]
struct WidgetData {
    start: Variable,
    end: Variable,
    pred: Option<LayoutId>,
    succ: Option<LayoutId>,
}
pub struct LinearLayoutHandler {
    pub padding: f32,

    orientation: Orientation,
    top: Variable,
    bottom: Variable,
    end: Option<Constraint>,

    widgets: HashMap<LayoutId, WidgetData>,
    last_widget: Option<LayoutId>,
}

impl LinearLayoutHandler {
    pub fn new(orientation: Orientation, parent: &Layout) -> Self {
        LinearLayoutHandler {
            padding: 0.0,
            orientation: orientation,
            top: beginning(orientation, &parent.vars),
            bottom: ending(orientation, &parent.vars),
            end: None,
            widgets: HashMap::new(),
            last_widget: None,
        }
    }
    pub fn add_child_layout(&mut self, parent: &mut Layout, child: &mut Layout) {
        match self.orientation {
            Orientation::Horizontal => {
                child.add(constraints![
                    bound_top(parent).padding(self.padding),
                    bound_bottom(parent).padding(self.padding),
                ]);
            }
            Orientation::Vertical => {
                child.add(constraints![
                    bound_left(parent).padding(self.padding),
                    bound_right(parent).padding(self.padding),
                ]);
            }
        }
        let child_start = beginning(self.orientation, &child.vars);
        let child_end = ending(self.orientation, &child.vars);
        let end = if let Some(last_widget) = self.last_widget {
            let last_widget = self.widgets.get_mut(&last_widget).unwrap();
            last_widget.succ = Some(child.id);
            last_widget.end
        } else {
            self.top
        };
        let constraint = child_start - end | EQ(REQUIRED) | self.padding;
        parent.add(constraint);
        if let Some(end) = self.end.take() {
            parent.remove_constraint(end);
        }
        let constraint = self.bottom - child_end | EQ(REQUIRED) | self.padding;
        self.end = Some(constraint.clone());
        parent.add(constraint);
        if let Some(last_widget_id) = self.last_widget {
            self.widgets.insert(child.id, WidgetData {
                start: child_start,
                end: child_end,
                pred: Some(last_widget_id),
                succ: None,
            });
        } else {
            self.widgets.insert(child.id, WidgetData {
                start: child_start,
                end: child_end,
                pred: None,
                succ: None,
            });
        }
        self.last_widget = Some(child.id);
    }
    pub fn remove_child_layout(&mut self, parent: &mut Layout, child_id: LayoutId) {
        if let Some(widget_data) = self.widgets.remove(&child_id) {
            if let Some(last_widget_id) = self.last_widget {
                if last_widget_id == child_id {
                    self.last_widget = widget_data.pred;
                }
            }
            let pred_end = if let Some(pred) = widget_data.pred {
                let pred = self.widgets.get_mut(&pred).unwrap();
                pred.succ = widget_data.succ;
                pred.end
            } else {
                self.top
            };
            if let Some(succ) = widget_data.succ {
                let succ = self.widgets.get_mut(&succ).unwrap();
                succ.pred = widget_data.pred;
                let succ_start = succ.start;
                parent.add(pred_end - succ_start | EQ(STRONG) | self.padding);
            }
        }
    }
}

fn beginning(orientation: Orientation, layout: &LayoutVars) -> Variable {
    match orientation {
        Orientation::Horizontal => layout.left,
        Orientation::Vertical => layout.top,
    }
}
fn ending(orientation: Orientation, layout: &LayoutVars) -> Variable {
    match orientation {
        Orientation::Horizontal => layout.right,
        Orientation::Vertical => layout.bottom,
    }
}
