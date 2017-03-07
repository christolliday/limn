pub mod graph;
pub mod solver;

pub use self::graph::WidgetGraph;
pub use self::solver::LimnSolver;

use backend::Window;

use event::{Queue, Target};
use resources::WidgetId;

use widget::WidgetBuilder;

pub struct Ui {
    pub graph: WidgetGraph,
    pub solver: LimnSolver,
}

impl Ui {
    pub fn new(window: &mut Window, queue: &Queue) -> Self {
        let graph = WidgetGraph::new(window);
        let solver = LimnSolver::new(queue.clone());
        Ui {
            graph: graph,
            solver: solver,
        }
    }

    pub fn set_root(&mut self, root_widget: WidgetBuilder, window: &mut Window) {
        self.graph.set_root(root_widget, &mut self.solver);
        self.graph.resize_window_to_fit(&window, &mut self.solver);
    }

    pub fn layout_changed(&mut self, event: &LayoutChanged, queue: &mut Queue) {
        let &LayoutChanged(widget_id) = event;
        if let Some(widget) = self.graph.get_widget(widget_id) {
            widget.layout.update(&mut self.solver);
        }
        // redraw everything when layout changes, for now
        queue.push(Target::Ui, RedrawEvent);
        self.graph.redraw();
    }
}

pub struct EventArgs<'a> {
    pub ui: &'a mut Ui,
    pub queue: &'a mut Queue,
}

pub trait EventHandler<T> {
    fn handle(&mut self, event: &T, args: EventArgs);
}

pub struct RedrawEvent;
pub struct LayoutChanged(pub WidgetId);

pub struct RedrawHandler;
impl EventHandler<RedrawEvent> for RedrawHandler {
    fn handle(&mut self, _: &RedrawEvent, args: EventArgs) {
        args.ui.graph.redraw();
    }
}
pub struct LayoutChangeHandler;
impl EventHandler<LayoutChanged> for LayoutChangeHandler {
    fn handle(&mut self, event: &LayoutChanged, args: EventArgs) {
        args.ui.layout_changed(event, args.queue);
    }
}
