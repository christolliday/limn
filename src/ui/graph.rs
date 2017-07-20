use std::collections::{HashMap, HashSet, VecDeque};

use widget::WidgetRef;
use util::Point;
use resources::WidgetId;

pub struct WidgetGraph {
    pub root: Option<WidgetRef>,
    widget_map: HashMap<WidgetId, WidgetRef>,
}
impl WidgetGraph {
    pub fn new() -> Self {
        WidgetGraph {
            widget_map: HashMap::new(),
            root: None,
        }
    }

    pub fn get_widget(&mut self, widget_id: WidgetId) -> Option<WidgetRef> {
        self.widget_map.get(&widget_id).map(|widget| widget.clone())
    }

    pub fn add_widget(&mut self, widget: WidgetRef) {
        self.widget_map.insert(widget.id(), widget);
    }

    pub fn remove_widget(&mut self, widget_id: WidgetId) {
        self.widget_map.remove(&widget_id);
    }
    pub fn get_root(&mut self) -> WidgetRef {
        self.root.as_ref().unwrap().clone()
    }

    pub fn widgets_under_cursor(&mut self, point: Point) -> CursorWidgetWalker {
        CursorWidgetWalker::new(point, self.get_root())
    }
    pub fn bfs(&mut self) -> Bfs {
        Bfs::new(self.get_root())
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
