use cassowary::strength::*;

use limn_layout::linear_layout::{LinearLayoutHandler, Orientation};
use limn_layout::grid_layout::GridLayout;
use limn_layout::constraint::*;

use resources::WidgetId;

use app::App;

use widget::{WidgetRef, WidgetBuilder};
use event::{EventHandler, EventArgs};
use ui::ChildrenUpdatedEvent;

pub use self::solver::LimnSolver;
pub use limn_layout::*;

impl EventHandler<ChildrenUpdatedEvent> for LinearLayoutHandler {
    fn handle(&mut self, event: &ChildrenUpdatedEvent, args: EventArgs) {
        args.widget.update_layout(|layout| {
            match *event {
                ChildrenUpdatedEvent::Added(ref child) => {
                    let child_id = child.id().0;
                    child.update_layout(|child_layout| {
                        self.add_child_layout(&layout.vars, child_layout, child_id);
                    });
                },
                ChildrenUpdatedEvent::Removed(ref child) => {
                    self.remove_child_layout(layout, child.id().0);
                },
            }
        });
    }
}

impl EventHandler<ChildrenUpdatedEvent> for GridLayout {
    fn handle(&mut self, event: &ChildrenUpdatedEvent, args: EventArgs) {
        args.widget.update_layout(|layout| {
            match *event {
                ChildrenUpdatedEvent::Added(ref child) => {
                    child.update_layout(|child_layout| {
                        self.add_child_layout(layout, child_layout);
                    });
                },
                ChildrenUpdatedEvent::Removed(_) => (),
            }
        });
    }
}

impl WidgetBuilder {
    pub fn vbox(&mut self) -> &mut Self {
        let handler = LinearLayoutHandler::new(Orientation::Vertical, &self.layout().vars);
        self.set_container(handler)
    }
    pub fn hbox(&mut self, padding: f32) -> &mut Self {
        let mut handler = LinearLayoutHandler::new(Orientation::Horizontal, &self.layout().vars);
        handler.padding = padding;
        self.set_container(handler)
    }
    pub fn grid(&mut self, num_columns: usize) {
        use std::ops::DerefMut;
        let container = GridLayout::new(self.layout().deref_mut(), num_columns);
        self.set_container(container);
    }
}

#[derive(Default)]
pub struct Frame {
    padding: f32,
}

impl EventHandler<ChildrenUpdatedEvent> for Frame {
    fn handle(&mut self, event: &ChildrenUpdatedEvent, args: EventArgs) {
        match *event {
            ChildrenUpdatedEvent::Added(ref child) => {
                child.update_layout(|layout| {
                    layout.add(constraints![
                        bound_by(&args.widget).padding(self.padding),
                        match_layout(&args.widget).strength(STRONG),
                    ]);
                });
            },
            ChildrenUpdatedEvent::Removed(_) => (),
        }
    }
}


/// wrapper around cassowary solver that keeps widgets positions in sync, sends events when layout changes happen
pub(super) struct LayoutManager {
    pub solver: LimnSolver,
}

impl LayoutManager {
    pub fn new() -> Self {
        LayoutManager {
            solver: LimnSolver::new(),
        }
    }
    /* pub fn check_changes(&mut self) {
        let changes = self.solver.fetch_changes();
        debug!("layout has {} changes", changes.len());
        if !changes.is_empty() {
            ui.event(LayoutChanged(changes));
        }
    } */
}

#[derive(Clone)]
pub struct UpdateLayout(pub WidgetRef);
pub struct ResizeWindow;
pub struct LayoutChanged(pub Vec<(usize, VarType, f64)>);
pub struct LayoutUpdated;

impl App {
    pub fn add_layout_handlers(&mut self) {
        self.add_handler_fn(|_: &ResizeWindow, args| {
            args.ui.resize_window_to_fit();
        });
        self.add_handler_fn(|event: &UpdateLayout, args| {
            let event = event.clone();
            let UpdateLayout(widget_ref) = event;
            let mut widget_mut = widget_ref.widget_mut();
            let layout = &mut widget_mut.layout;
            args.ui.layout.solver.update_layout(layout);
            args.ui.check_layout_changes();
        });
        self.add_handler_fn(|event: &LayoutChanged, args| {
            let changes = &event.0;
            for &(widget_id, var, value) in changes {
                let widget_id = WidgetId(widget_id);
                if let Some(widget) = args.ui.get_widget(widget_id) {
                    {
                        let widget = &mut *widget.widget_mut();
                        let value = value as f32;
                        debug!("{:?}: {:?} = {}", widget.name(), var, value);
                        match var {
                            VarType::Left => widget.bounds.origin.x = value,
                            VarType::Top => widget.bounds.origin.y = value,
                            VarType::Width => widget.bounds.size.width = value,
                            VarType::Height => widget.bounds.size.height = value,
                            _ => (),
                        }
                    }
                    widget.event(LayoutUpdated);
                }
            }
            // redraw everything when layout changes, for now
            args.ui.redraw();
        });
    }
}
