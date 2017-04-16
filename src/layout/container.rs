use layout::LayoutVars;
use widget::{WidgetBuilder, WidgetBuilderCore};

pub trait LayoutContainer {
    fn add_child(&mut self, parent: &LayoutVars, child: &mut WidgetBuilder);
}

pub struct Frame;
impl LayoutContainer for Frame {
    fn add_child(&mut self, parent: &LayoutVars, child: &mut WidgetBuilder) {
        child.layout().bound_by(parent);
    }
}