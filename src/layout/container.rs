use layout::LayoutVars;
use layout::solver::LimnSolver;
use widget::{Widget, WidgetBuilder, WidgetBuilderCore};
use resources::WidgetId;

pub trait LayoutContainer {
    fn set_padding(&mut self, padding: f64);
    fn add_child(&mut self, parent: &LayoutVars, child: &mut WidgetBuilder);
    fn remove_child(&mut self, parent: &Widget, child_id: WidgetId, solver: &mut LimnSolver);
}

pub struct Frame {
    padding: Option<f64>,
}
impl Frame {
    pub fn new() -> Self {
        Frame {
            padding: None,
        }
    }
}
impl LayoutContainer for Frame {
    fn set_padding(&mut self, padding: f64) {
        self.padding = Some(padding);
    }
    fn add_child(&mut self, parent: &LayoutVars, child: &mut WidgetBuilder) {
        let constraint = child.layout().bound_by(parent);
        if let Some(padding) = self.padding {
            constraint.padding(padding);
        }
    }
    fn remove_child(&mut self, _: &Widget, _: WidgetId, _: &mut LimnSolver) {}
}