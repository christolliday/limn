use cassowary::Variable;

use limn_layout::linear_layout::{LinearLayoutHandler, Orientation};
use limn_layout::grid_layout::GridLayout;
use limn_layout::solver::VarType;

use resources::WidgetId;

use app::App;
use event::Target;

use widget::Widget;

use self::container::LayoutContainer;

pub mod container;

pub use self::solver::LimnSolver;
pub use limn_layout::*;

impl LayoutContainer for LinearLayoutHandler {
    fn set_padding(&mut self, padding: f32) {
        self.padding = padding;
    }
    fn add_child(&mut self, mut parent: Widget, mut child: Widget) {
        let child_id = child.id();
        parent.update_layout(|layout| {
            self.add_child_layout(&layout.vars, &mut child.layout(), child_id.0);
        });
    }
    fn remove_child(&mut self, mut parent: Widget, child_id: WidgetId) {
        parent.update_layout(|layout| {
            self.remove_child_layout(layout, child_id.0);
        });
    }
}

impl LayoutContainer for GridLayout {
    fn add_child(&mut self, mut parent: Widget, mut child: Widget) {
        parent.update_layout(|layout| {
            self.add_child_layout(layout, &mut child.layout());
        });
    }
}

impl Widget {
    pub fn vbox(&mut self) -> &mut Self {
        let handler = LinearLayoutHandler::new(Orientation::Vertical, &self.layout().vars);
        self.set_container(handler)
    }
    pub fn hbox(&mut self) -> &mut Self {
        let handler = LinearLayoutHandler::new(Orientation::Horizontal, &self.layout().vars);
        self.set_container(handler)
    }
    pub fn grid(&mut self, num_columns: usize) {
        use std::ops::DerefMut;
        let container = GridLayout::new(self.layout().deref_mut(), num_columns);
        self.set_container(container);
    }
}


/// wrapper around cassowary solver that keeps widgets positions in sync, sends events when layout changes happen
pub struct LayoutManager {
    pub solver: LimnSolver,
}

impl LayoutManager {
    pub fn new() -> Self {
        LayoutManager {
            solver: LimnSolver::new(),
        }
    }
    pub fn update_solver<F>(&mut self, f: F)
        where F: Fn(&mut LimnSolver)
    {
        f(&mut self.solver);
        self.check_changes();
    }

    pub fn check_changes(&mut self) {
        let changes = self.solver.fetch_changes();
        debug!("layout has {} changes", changes.len());
        if !changes.is_empty() {
            event!(Target::Ui, LayoutChanged(changes));
        }
    }
}

#[derive(Clone)]
pub struct UpdateLayout(pub Widget);
pub struct ResizeWindow;
pub struct LayoutChanged(Vec<(usize, VarType, f64)>);
pub struct LayoutUpdated;

impl App {
    pub fn add_layout_handlers(&mut self) {
        self.add_handler_fn(|_: &ResizeWindow, ui| {
            ui.resize_window_to_fit();
        });
        self.add_handler_fn(|event: &UpdateLayout, ui| {
            let event = event.clone();
            let UpdateLayout(mut widget_ref) = event;
            ui.layout.solver.update_layout(&mut widget_ref.layout());
            ui.layout.check_changes();
        });
        self.add_handler_fn(|event: &LayoutChanged, ui| {
            let changes = &event.0;
            for &(widget_id, var, value) in changes {
                let widget_id = WidgetId(widget_id);
                if let Some(widget) = ui.get_widget(widget_id) {
                    //let vars = &ui.layout.solver.layouts[&widget_id.0];
                    {
                        let widget = &mut *widget.widget_mut();
                        //let var = vars.get_var(var).expect("Missing variable for widget");
                        let value = value as f32;
                        debug!("{:?}: {:?} = {}", widget.debug_name, var, value);
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
            ui.redraw();
        });
    }
}
