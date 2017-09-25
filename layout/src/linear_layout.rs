use std::collections::HashMap;

use cassowary::strength::*;
use cassowary::WeightedRelation::*;
use cassowary::{Variable, Constraint};

use super::{LayoutId, LayoutVars, Layout, LayoutContainer};
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
    end_constraint: Option<Constraint>,
}
pub struct LinearLayout {
    orientation: Orientation,
    padding: f32,

    start: Variable,
    end: Variable,
    space: Variable,

    widgets: HashMap<LayoutId, WidgetData>,
    last_widget: Option<LayoutId>,
}

impl LinearLayout {
    pub fn new(parent: &mut Layout, orientation: Orientation, padding: f32, expand: bool) -> Self {
        let start = Variable::new();
        let end = Variable::new();
        let space = Variable::new();
        let parent_start = beginning(orientation, &parent.vars);
        let parent_end = ending(orientation, &parent.vars);
        parent.add(constraints![
            start | GE(REQUIRED) | parent_start + padding,
            end | LE(REQUIRED) | parent_end - padding,
            space | EQ(REQUIRED) | start - parent_start,
        ]);
        if expand {
            parent.add(parent_end - end | EQ(REQUIRED) | space);
        } else {
            parent.add(parent_end - end | EQ(STRONG) | parent_end - parent_start);
        }
        parent.add_associated_var(start, "linear_layout_start");
        parent.add_associated_var(end, "linear_layout_end");
        parent.add_associated_var(space, "linear_layout_space");
        LinearLayout {
            orientation: orientation,
            padding: padding,
            start: start,
            end: end,
            space: space,
            widgets: HashMap::new(),
            last_widget: None,
        }
    }
}

impl LayoutContainer for LinearLayout {
    fn add_child_layout(&mut self, parent: &mut Layout, child: &mut Layout) {

        let child_start = beginning(self.orientation, &child.vars);
        let child_end = ending(self.orientation, &child.vars);

        if let Some(last_id) = self.last_widget {
            let last_widget = self.widgets.get_mut(&last_id).unwrap();
            parent.remove_constraint(last_widget.end_constraint.take().unwrap());
            parent.add(constraints![
                child_start | GE(REQUIRED) | last_widget.end + self.padding,
                child_start - last_widget.end | EQ(REQUIRED) | self.space,
            ]);
            last_widget.succ = Some(child.id);
        } else {
            parent.add(constraints![
                child_start | EQ(REQUIRED) | self.start,
            ]);
        }
        let end_constraint = child_end | EQ(REQUIRED) | self.end;
        parent.add(end_constraint.clone());
        self.widgets.insert(child.id, WidgetData {
            start: child_start,
            end: child_end,
            pred: self.last_widget,
            succ: None,
            end_constraint: Some(end_constraint),
        });
        self.last_widget = Some(child.id);

        match self.orientation {
            Orientation::Horizontal => {
                child.add(constraints![
                    bound_top(parent),
                    bound_bottom(parent),
                ]);
            }
            Orientation::Vertical => {
                child.add(constraints![
                    bound_left(parent),
                    bound_right(parent),
                ]);
            }
        }
    }
    fn remove_child_layout(&mut self, parent: &mut Layout, child: &mut Layout) {
        if let Some(widget_data) = self.widgets.remove(&child.id) {
            if let Some(last_widget_id) = self.last_widget {
                if last_widget_id == child.id {
                    self.last_widget = widget_data.pred;
                }
            }
            let pred_end = if let Some(pred) = widget_data.pred {
                let pred = self.widgets.get_mut(&pred).unwrap();
                pred.succ = widget_data.succ;
                pred.end
            } else {
                self.start
            };
            if let Some(succ) = widget_data.succ {
                let succ = self.widgets.get_mut(&succ).unwrap();
                succ.pred = widget_data.pred;
                let succ_start = succ.start;
                parent.add(constraints![
                    succ_start | GE(REQUIRED) | pred_end + self.padding,
                    succ_start - pred_end | EQ(REQUIRED) | self.space,
                ]);
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
