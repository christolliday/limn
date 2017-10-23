use std::ops::DerefMut;

use limn_layout::linear_layout::{LinearLayout, LinearLayoutSettings};
use limn_layout::grid_layout::GridLayout;

use resources::WidgetId;

use app::App;

use widget::{WidgetRef, WidgetBuilder};

pub use self::solver::LimnSolver;
pub use limn_layout::*;

impl WidgetBuilder {

    /// Creates a linear layout with Widgets positioned relatively to each other
    pub fn linear_layout(&mut self, settings: LinearLayoutSettings) -> &mut Self {
        let container = LinearLayout::new(self.layout().deref_mut(), settings);
        self.layout().set_container(container);
        self
    }

    /// Creates a grid-based layout with a certain number of columns
    pub fn grid(&mut self, num_columns: usize) -> &mut Self {
        let container = GridLayout::new(self.layout().deref_mut(), num_columns);
        self.layout().set_container(container);
        self
    }
}

#[derive(Clone)]
pub struct UpdateLayout(pub WidgetRef);
#[derive(Debug, Copy, Clone)]
pub struct ResizeWindow;
pub struct LayoutChanged(pub Vec<(usize, VarType, f64)>);
#[derive(Debug, Copy, Clone)]
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
            args.ui.solver.update_layout(layout);
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
