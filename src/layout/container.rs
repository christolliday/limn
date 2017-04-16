use layout::LayoutVars;
use layout::solver::LimnSolver;
use widget::{Widget, WidgetBuilder, WidgetBuilderCore};
use resources::WidgetId;

pub trait LayoutContainer {
    fn add_child(&mut self, parent: &LayoutVars, child: &mut WidgetBuilder);
    fn remove_child(&mut self, parent: &Widget, child_id: WidgetId, solver: &mut LimnSolver);
}

pub struct Frame;
impl LayoutContainer for Frame {
    fn add_child(&mut self, parent: &LayoutVars, child: &mut WidgetBuilder) {
        child.layout().bound_by(parent);
    }
    fn remove_child(&mut self, _: &Widget, _: WidgetId, _: &mut LimnSolver) {}
}