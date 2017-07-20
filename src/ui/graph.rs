use std::collections::{HashMap, HashSet, VecDeque};

use petgraph::stable_graph::StableGraph;
use petgraph::graph::NodeIndex;

use widget::WidgetRef;
use util::Point;
use resources::{resources, WidgetId};

type Graph = StableGraph<WidgetRef, ()>;

/**
Most of the functionality of WidgetGraph is to wrap NodeIndex so only WidgetId is exposed to the Ui.
WidgetId is used outside of this class so that new WidgetIds can be generated
without holding a mutable reference to the graph. This greatly simplifies the ergonomics
of creating widgets via WidgetBuilder.
Conrod's solution (to a slightly different problem) is to pre-generate the Widget Ids, before modifying
the widgets, but that also has cost in terms of ergonomics, especially when you want anonymous Widget Ids.

A better long term solution could be to extend petgraphs stable_graph to have a mutable Index generator that
the graph references, that can potentially outlive the graph and can be assumed to not be tied to a single
graph, but that generates NodeIndexes directly, so that WidgetId can be an alias for NodeIndex.
That generator could be owned by and be accessed via the global Mutex in Resources, while the graph itself
is owned by the Ui.
*/
pub struct WidgetGraph {
    pub graph: Graph,
    pub root_id: WidgetId,
    pub root: Option<WidgetRef>,
    widget_map: HashMap<WidgetId, NodeIndex>,
}
impl WidgetGraph {
    pub fn new() -> Self {
        WidgetGraph {
            graph: Graph::new(),
            widget_map: HashMap::new(),
            root_id: resources().widget_id(),
            root: None,
        }
    }

    pub fn get_widget(&mut self, widget_id: WidgetId) -> Option<WidgetRef> {
        if let Some(node_index) = self.widget_map.get(&widget_id) {
            if let Some(widget_container) = self.graph.node_weight_mut(node_index.clone()) {
                return Some(widget_container.clone())
            }
        }
        None
    }

    pub fn add_widget(&mut self,
                      widget: WidgetRef,
                      parent_id: Option<WidgetId>)
                      -> NodeIndex
    {
        let id = widget.id();
        let widget_index = self.graph.add_node(widget);
        self.widget_map.insert(id, widget_index);
        if let Some(parent_id) = parent_id {
            if let Some(parent_index) = self.find_widget(parent_id) {
                self.graph.add_edge(parent_index, widget_index, ());
            }
        }
        widget_index
    }

    pub fn remove_widget(&mut self, widget_id: WidgetId) -> Option<WidgetRef> {
        if let Some(node_index) = self.find_widget(widget_id) {
            self.widget_map.remove(&widget_id);
            if let Some(widget) = self.graph.remove_node(node_index) {
                return Some(widget);
            }
        }
        None
    }
    fn find_widget(&self, widget_id: WidgetId) -> Option<NodeIndex> {
        self.widget_map.get(&widget_id).map(|index| *index)
    }
    pub fn get_root(&mut self) -> WidgetRef {
        self.root.as_ref().unwrap().clone()
    }

    pub fn widgets_under_cursor(&mut self, point: Point) -> CursorWidgetWalker {
        CursorWidgetWalker::new(point, self.get_root())
    }
    pub fn bfs(&mut self, widget_id: WidgetId) -> Bfs {
        let widget_ref = self.get_widget(widget_id).unwrap();
        Bfs::new(widget_ref)
    }
}

pub struct CursorWidgetWalker {
    point: Point,
    dfs: DfsPostReverse,
}
impl CursorWidgetWalker {
    fn new(point: Point, root: WidgetRef) -> Self {
        CursorWidgetWalker {
            point: point,
            dfs: DfsPostReverse::new(root),
        }
    }
    pub fn next(&mut self) -> Option<WidgetId> {
        while let Some(widget_ref) = self.dfs.next() {
            let ref widget_container = widget_ref.widget_container();
            if widget_container.widget.is_mouse_over(self.point) {
                return Some(widget_container.widget.id);
            }
        }
        None
    }
}

// iterates in reverse of draw order, that is, depth first post order,
// with siblings in reverse of insertion order
pub struct DfsPostReverse {
    stack: Vec<WidgetRef>,
    discovered: HashSet<WidgetRef>,
    finished: HashSet<WidgetRef>,
}

impl DfsPostReverse {
    fn new(root: WidgetRef) -> Self {
        DfsPostReverse {
            stack: vec![root],
            discovered: HashSet::new(),
            finished: HashSet::new(),
        }
    }
    pub fn next(&mut self) -> Option<WidgetRef> {
        while let Some(widget_ref) = self.stack.last().map(|w| w.clone()) {
            if self.discovered.insert(widget_ref.clone()) {
                for child in &widget_ref.widget_container().widget.children {
                    self.stack.push(child.clone());
                }
            } else {
                self.stack.pop();
                if self.finished.insert(widget_ref.clone()) {
                    return Some(widget_ref.clone());
                }
            }
        }
        None
    }
}

pub struct Bfs {
    queue: VecDeque<WidgetRef>,
}

impl Bfs {
    fn new(root: WidgetRef) -> Self {
        let mut queue = VecDeque::new();
        queue.push_front(root);
        Bfs { queue: queue }
    }
    pub fn next(&mut self) -> Option<WidgetRef> {
        while let Some(widget_ref) = self.queue.pop_front() {
            for child in &widget_ref.widget_container().widget.children {
                self.queue.push_back(child.clone());
            }
            return Some(widget_ref);
        }
        None
    }
}
