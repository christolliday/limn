pub mod graph;
pub mod solver;

pub use self::graph::WidgetGraph;
pub use self::solver::LimnSolver;

use backend::Window;

use event::Queue;

use widget::WidgetBuilder;

pub struct Ui {
    pub graph: WidgetGraph,
    pub solver: LimnSolver,
    should_close: bool,
}

impl Ui {
    pub fn new(window: &mut Window, queue: &Queue) -> Self {
        let graph = WidgetGraph::new(window);
        let solver = LimnSolver::new(queue.clone());
        Ui {
            graph: graph,
            solver: solver,
            should_close: false,
        }
    }
    pub fn close(&mut self) {
        self.should_close = true;
    }
    pub fn should_close(&self) -> bool {
        self.should_close
    }
    pub fn set_root(&mut self, root_widget: WidgetBuilder) {
        let root_widget = root_widget.set_debug_name("root");
        self.graph.set_root(root_widget, &mut self.solver);
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

pub struct RedrawHandler;
impl EventHandler<RedrawEvent> for RedrawHandler {
    fn handle(&mut self, _: &RedrawEvent, args: EventArgs) {
        args.ui.graph.redraw();
    }
}
