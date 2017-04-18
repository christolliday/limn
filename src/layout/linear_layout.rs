use std::collections::HashMap;

use cassowary::strength::*;
use cassowary::WeightedRelation::*;
use cassowary::Variable;

use widget::{Widget, WidgetBuilder, WidgetBuilderCore};
use layout::constraint::*;
use layout::LayoutVars;
use layout::solver::LimnSolver;
use layout::container::LayoutContainer;
use resources::WidgetId;

#[derive(Copy, Clone)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Debug)]
struct WidgetData {
    start: Variable,
    end: Variable,
    pred: Option<WidgetId>,
    succ: Option<WidgetId>,
}
struct LinearLayoutHandler {
    padding: f64,

    orientation: Orientation,
    top: Variable,
    bottom: Variable,

    widgets: HashMap<WidgetId, WidgetData>,
    last_widget: Option<WidgetId>,
}

impl LinearLayoutHandler {
    fn new(orientation: Orientation, parent: &LayoutVars) -> Self {
        LinearLayoutHandler {
            padding: 0.0,
            orientation: orientation,
            top: beginning(orientation, parent),
            bottom: ending(orientation, parent),
            widgets: HashMap::new(),
            last_widget: None,
        }
    }
}

impl LayoutContainer for LinearLayoutHandler {
    fn set_padding(&mut self, padding: f64) {
        self.padding = padding;
    }
    fn add_child(&mut self, parent: &LayoutVars, child: &mut WidgetBuilder) {
        match self.orientation {
            Orientation::Horizontal => {
                layout!(child:
                    bound_top(parent).padding(self.padding),
                    bound_bottom(parent).padding(self.padding));
            }
            Orientation::Vertical => {
                layout!(child:
                    bound_left(parent).padding(self.padding),
                    bound_right(parent).padding(self.padding));
            }
        }
        let child_start = beginning(self.orientation, &child.layout().vars);
        let child_end = ending(self.orientation, &child.layout().vars);
        let end = if let Some(last_widget) = self.last_widget {
            let last_widget = self.widgets.get_mut(&last_widget).unwrap();
            last_widget.succ = Some(child.id());
            last_widget.end
        } else {
            self.top
        };
        let constraint = child_start - end | EQ(REQUIRED) | self.padding;
        child.layout().constraints.push(constraint);
        let constraint = self.bottom - child_end | GE(REQUIRED) | self.padding;
        child.layout().constraints.push(constraint);
        if let Some(last_widget_id) = self.last_widget {
            self.widgets.insert(child.id(), WidgetData {
                start: child_start,
                end: child_end,
                pred: Some(last_widget_id),
                succ: None,
            });
        } else {
            self.widgets.insert(child.id(), WidgetData {
                start: child_start,
                end: child_end,
                pred: None,
                succ: None,
            });
        }
        self.last_widget = Some(child.id());
    }
    fn remove_child(&mut self, parent: &Widget, child_id: WidgetId, solver: &mut LimnSolver) {
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
                let padding = self.padding;
                parent.update_layout(|layout| {
                    let constraint = pred_end - succ_start | EQ(STRONG) | padding;
                    layout.constraints.push(constraint);
                }, solver);
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

impl WidgetBuilder {
    pub fn vbox(&mut self) -> &mut Self {
        let handler = LinearLayoutHandler::new(Orientation::Vertical, &self.layout().vars);
        self.set_container(handler)
    }
    pub fn hbox(&mut self) -> &mut Self {
        let handler = LinearLayoutHandler::new(Orientation::Horizontal, &self.layout().vars);
        self.set_container(handler)
    }
}