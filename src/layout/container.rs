use limn_layout::constraint::*;
use widget::WidgetRef;
use resources::WidgetId;

pub trait LayoutContainer {
    fn set_padding(&mut self, _padding: f64) {}
    fn add_child(&mut self, parent: WidgetRef, child: WidgetRef);
    fn remove_child(&mut self, _parent: WidgetRef, _child_id: WidgetId) {}
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
    fn add_child(&mut self, parent: WidgetRef, mut child: WidgetRef) {
        layout!(child: bound_by(&parent).padding(self.padding));
    }
}
