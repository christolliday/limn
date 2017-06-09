use limn_layout::constraint::*;
use layout::LayoutManager;
use widget::{Widget, WidgetBuilder, WidgetBuilderCore};
use resources::WidgetId;

pub trait LayoutContainer {
    fn set_padding(&mut self, _padding: f64) {}
    fn add_child(&mut self, parent: &mut Widget, child: &mut WidgetBuilder);
    fn remove_child(&mut self, _parent: &mut Widget, _child_id: WidgetId, _solver: &mut LayoutManager) {}
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
    fn add_child(&mut self, parent: &mut Widget, child: &mut WidgetBuilder) {
        let ref parent = parent.layout;
        layout!(child: bound_by(parent).padding(self.padding));
    }
}
