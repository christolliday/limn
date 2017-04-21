use layout::solver::LimnSolver;
use layout::constraint::*;
use widget::{Widget, WidgetBuilder, WidgetBuilderCore};
use resources::WidgetId;

pub trait LayoutContainer {
    fn set_padding(&mut self, padding: f64);
    fn add_child(&mut self, parent: &Widget, child: &mut WidgetBuilder);
    fn remove_child(&mut self, parent: &Widget, child_id: WidgetId, solver: &mut LimnSolver);
}

pub struct Frame {
    padding: f64,
}
impl Frame {
    pub fn new() -> Self {
        Frame {
            padding: 0.0,
        }
    }
}
impl LayoutContainer for Frame {
    fn set_padding(&mut self, padding: f64) {
        self.padding = padding;
    }
    fn add_child(&mut self, parent: &Widget, child: &mut WidgetBuilder) {
        let ref parent = parent.layout;
        layout!(child: bound_by(parent).padding(self.padding));
    }
    fn remove_child(&mut self, _: &Widget, _: WidgetId, _: &mut LimnSolver) {}
}