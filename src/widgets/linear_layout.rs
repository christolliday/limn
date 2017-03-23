use std::collections::HashMap;

use cassowary::strength::*;
use cassowary::WeightedRelation::*;
use cassowary::Variable;

use widget::{WidgetBuilder, EventHandler, EventArgs};
use widget::layout::LayoutVars;
use ui::graph::ChildAttachedEvent;
use resources::WidgetId;
use event::Target;

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
    orientation: Orientation,
    top: Variable,

    widgets: HashMap<WidgetId, WidgetData>,
    last_widget: Option<WidgetId>,
}

impl LinearLayoutHandler {
    fn new(orientation: Orientation, parent: &LayoutVars) -> Self {
        LinearLayoutHandler {
            orientation: orientation,
            top: beginning(orientation, parent),
            widgets: HashMap::new(),
            last_widget: None,
        }
    }
}

pub enum LinearLayoutEvent {
    AddWidget(WidgetId, LayoutVars),
    RemoveWidget(WidgetId),
}
impl EventHandler<LinearLayoutEvent> for LinearLayoutHandler {
    fn handle(&mut self, event: &LinearLayoutEvent, args: EventArgs) {
        match *event {
            LinearLayoutEvent::AddWidget(child_id, ref child_layout) => {

                let child_start = beginning(self.orientation, &child_layout);
                let child_end = ending(self.orientation, &child_layout);

                let end = if let Some(last_widget) = self.last_widget {
                    let last_widget = self.widgets.get_mut(&last_widget).unwrap();
                    last_widget.succ = Some(child_id);
                    last_widget.end
                } else {
                    self.top
                };
                args.widget.update_layout(|layout| {
                    let constraint = child_start | EQ(REQUIRED) | end;
                    layout.constraints.push(constraint);
                }, args.solver);
                if let Some(last_widget_id) = self.last_widget {
                    self.widgets.insert(child_id, WidgetData {
                        start: child_start,
                        end: child_end,
                        pred: Some(last_widget_id),
                        succ: None,
                    });
                } else {
                    self.widgets.insert(child_id, WidgetData {
                        start: child_start,
                        end: child_end,
                        pred: None,
                        succ: None,
                    });
                }
                self.last_widget = Some(child_id);
            },
            LinearLayoutEvent::RemoveWidget(widget_id) => {
                if let Some(widget_data) = self.widgets.remove(&widget_id) {

                    if let Some(last_widget_id) = self.last_widget {
                        if last_widget_id == widget_id {
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
                        args.widget.update_layout(|layout| {
                            let constraint = pred_end | EQ(REQUIRED) | succ_start;
                            layout.constraints.push(constraint);
                        }, args.solver);
                    }
                }
            },
        }
    }
}

struct LinearLayoutChildAttachedHandler;
impl EventHandler<ChildAttachedEvent> for LinearLayoutChildAttachedHandler {
    fn handle(&mut self, event: &ChildAttachedEvent, args: EventArgs) {
        let &ChildAttachedEvent(child_id, ref child_layout) = event;
        args.queue.push(Target::Widget(args.widget.id), LinearLayoutEvent::AddWidget(child_id, child_layout.clone()));
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
        let handler = LinearLayoutHandler::new(Orientation::Vertical, &self.layout.vars);
        self.add_handler(handler);
        self.add_handler(LinearLayoutChildAttachedHandler)
    }
    pub fn hbox(&mut self) -> &mut Self {
        let handler = LinearLayoutHandler::new(Orientation::Horizontal, &self.layout.vars);
        self.add_handler(handler);
        self.add_handler(LinearLayoutChildAttachedHandler)
    }
}