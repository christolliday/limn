use cassowary::strength::*;
use cassowary::WeightedRelation::*;
use cassowary::Variable;

use widget::{WidgetBuilder, EventHandler, EventArgs};
use widget::layout::LayoutVars;
use ui::graph::ChildAttachedEvent;

#[derive(Copy, Clone)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

pub struct LinearLayoutHandler {
    pub orientation: Orientation,
    pub end: Variable,
}
impl LinearLayoutHandler {
    pub fn new(orientation: Orientation, parent: &LayoutVars) -> Self {
        LinearLayoutHandler {
            orientation: orientation,
            end: beginning(orientation, parent),
        }
    }
}

impl EventHandler<ChildAttachedEvent> for LinearLayoutHandler {
    fn handle(&mut self, event: &ChildAttachedEvent, args: EventArgs) {
        let &ChildAttachedEvent(ref child_layout) = event;
        args.widget.update_layout(|layout| {
            let constraint = beginning(self.orientation, &child_layout) | GE(REQUIRED) | self.end;
            self.end = ending(self.orientation, &child_layout);
            layout.constraints.push(constraint);
        }, args.solver);
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
        self.add_handler(handler)
    }
    pub fn hbox(&mut self) -> &mut Self {
        let handler = LinearLayoutHandler::new(Orientation::Horizontal, &self.layout.vars);
        self.add_handler(handler)
    }
}