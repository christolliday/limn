use cassowary::strength::*;

use widget::WidgetRef;
use resources::WidgetId;
use layout::constraint::*;

pub trait LayoutContainer {
    fn set_padding(&mut self, _padding: f32) {}
    fn add_child(&mut self, parent: WidgetRef, child: WidgetRef);
    fn remove_child(&mut self, _parent: WidgetRef, _child_id: WidgetId) {}
}

pub struct Frame {
    padding: f32,
}
impl Frame {
    pub fn new() -> Self {
        Frame {
            padding: 0.0,
        }
    }
}
impl LayoutContainer for Frame {
    fn set_padding(&mut self, padding: f32) {
        self.padding = padding;
    }
    fn add_child(&mut self, parent: WidgetRef, mut child: WidgetRef) {
        child.update_layout(|layout| {
            layout.add(constraints![
                bound_by(&parent).padding(self.padding),
                match_layout(&parent).strength(STRONG),
            ]);
        });
    }
}
