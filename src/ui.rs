
use backend::gfx::G2d;

use petgraph::Graph;
use petgraph::graph::NodeIndex;
use petgraph::visit::Dfs;

use input::{Event, GenericEvent, MouseCursorEvent, UpdateArgs};

use cassowary::{Solver, Constraint};
use cassowary::WeightedRelation::*;
use cassowary::strength::*;

use graphics::Context;
use super::widget::*;
use super::widget::primitives::EmptyDrawable;
use super::widget;
use super::util::*;
use super::event;
use resources;
use resources::font::Font;
use backend::glyph::GlyphCache;
use backend::window::Window;

use resources::image::Texture;

const DEBUG_BOUNDS: bool = true;

pub struct Resources {
    pub glyph_cache: GlyphCache,
    pub fonts: resources::Map<Font>,
    pub images: resources::Map<Texture>,
}
impl Resources {
    fn new(glyph_cache: GlyphCache) -> Self {
        let fonts = resources::Map::new();
        let images = resources::Map::new();
        Resources {
            fonts: fonts,
            images: images,
            glyph_cache: glyph_cache,
        }
    }
}

pub struct InputState {
    pub mouse: Point,
}
impl InputState {
    fn new() -> Self {
        InputState { mouse: Point { x: 0.0, y: 0.0 }}
    }
}

pub struct Ui {
    pub graph: Graph<Widget, ()>,
    pub root_index: NodeIndex,
    constraints: Vec<Constraint>,
    pub solver: Solver,
    pub resources: Resources,
    pub input_state: InputState,
}
impl Ui {
    pub fn new(window: &mut Window, window_dims: Dimensions) -> Self {
        let root = Widget::new();
        let mut constraints = Vec::new();
        let mut solver = Solver::new();

        let mut graph = Graph::<Widget, ()>::new();
        solver.add_edit_variable(root.layout.right, STRONG).unwrap();
        solver.add_edit_variable(root.layout.bottom, STRONG).unwrap();
        constraints.push(root.layout.left | EQ(STRONG) | 0.0);
        constraints.push(root.layout.top | EQ(STRONG) | 0.0);
        let root_index = graph.add_node(root);

        let glyph_cache = GlyphCache::new(&mut window.context.factory,
                                          window_dims.width as u32,
                                          window_dims.height as u32);

        let resources = Resources::new(glyph_cache);
        let input_state = InputState::new();
        let mut ui = Ui {
            graph: graph,
            root_index: root_index,
            solver: solver,
            constraints: constraints,
            resources: resources,
            input_state: input_state,
        };
        ui.resize_window(window_dims);
        ui
    }
    pub fn resize_window(&mut self, window_dims: Dimensions) {
        let ref root = self.graph[self.root_index];
        self.solver.suggest_value(root.layout.right, window_dims.width).unwrap();
        self.solver.suggest_value(root.layout.bottom, window_dims.height).unwrap();
    }
    pub fn init(&mut self) {
        /*let mut dfs = Dfs::new(&self.graph, self.root_index);
        while let Some(node_index) = dfs.next(&self.graph) {
            let ref mut node = self.graph[node_index];
            let constraints = &mut node.layout.constraints;
            self.constraints.append(constraints);
        }
        self.solver.add_constraints(&self.constraints).unwrap();*/
    }
    pub fn parent_index(&mut self, node_index: NodeIndex) -> Option<NodeIndex> {
        use petgraph::Direction;
        let mut neighbors = self.graph.neighbors_directed(node_index, Direction::Incoming);
        neighbors.next()
    }
    pub fn draw(&mut self, c: Context, g: &mut G2d) {
        let mut dfs = Dfs::new(&self.graph, self.root_index);
        while let Some(node_index) = dfs.next(&self.graph) {

            if let Some(parent_index) = self.parent_index(node_index) {
                let (parent, widget) = self.graph.index_twice_mut(parent_index, node_index);

                if DEBUG_BOUNDS {
                    draw_rect_outline(widget.layout.bounds(&mut self.solver),
                                    [0.0, 1.0, 1.0, 1.0],
                                    c,
                                    g);
                }
                widget.draw(&parent, &mut self.resources, &mut self.solver, c, g);
            }
        }
    }
    pub fn add_widget(&mut self, parent_index: NodeIndex, child: Widget) -> NodeIndex {
        let child_index = self.graph.add_node(child);
        self.graph.add_edge(parent_index, child_index, ());

        let (parent, child) = self.graph.index_twice_mut(parent_index, child_index);

        self.solver.add_constraints(&child.layout.constraints).unwrap();

        if parent.layout.scrollable {
            let child_bounds = child.layout.bounds(&mut self.solver);
            let parent_bounds = parent.layout.bounds(&mut self.solver);

            println!("{:?} {:?}", child_bounds, parent_bounds);

            let mut constraints = Vec::new();
            if child_bounds.width <= parent_bounds.width {
                constraints.push(child.layout.left | EQ(REQUIRED) | parent.layout.left);
            } else {
                self.solver.add_edit_variable(child.layout.left, STRONG).unwrap();
                self.solver.suggest_value(child.layout.left, parent_bounds.left);
                constraints.push(child.layout.left | LE(REQUIRED) | parent.layout.left);
                constraints.push(child.layout.right | GE(REQUIRED) | parent.layout.right);
            }
            if child_bounds.height <= parent_bounds.height {
                constraints.push(child.layout.top | EQ(REQUIRED) | parent.layout.top);
            } else {
                self.solver.add_edit_variable(child.layout.top, STRONG).unwrap();
                self.solver.suggest_value(child.layout.top, parent_bounds.top);
                constraints.push(child.layout.top | LE(REQUIRED) | parent.layout.top);
                constraints.push(child.layout.bottom | GE(REQUIRED) | parent.layout.bottom);
            }
            self.solver.add_constraints(&constraints).unwrap();

        } else {
            // TODO set these constraints
            child.layout.bound_by(&parent.layout);
            
        }

        child_index
    }
    pub fn handle_event(&mut self, event: &Event) {
        if let Some(mouse) = event.mouse_cursor_args() {
            self.input_state.mouse = mouse.into();
        }
        self.post_event(event);
    }
    pub fn post_event(&mut self, event: &Event) {
        let mut new_events = Vec::new();
        let id_registered = |widget: &Widget, id| { widget.event_handlers.iter().any(|event_handler| event_handler.event_id() == id) };
        
        let mut dfs = Dfs::new(&self.graph, self.root_index);
        while let Some(node_index) = dfs.next(&self.graph) {
            if let Some(parent_index) = self.parent_index(node_index) {
                let (parent, widget) = self.graph.index_twice_mut(parent_index, node_index);

                let is_mouse_over = widget.is_mouse_over(&mut self.solver, self.input_state.mouse);
                if is_mouse_over {
                    if event.event_id() == event::MOUSE_CURSOR && id_registered(widget, event::WIDGET_MOUSE_OVER) {
                        widget.trigger_event(event::WIDGET_MOUSE_OVER, event, &parent.layout, &mut self.solver);
                    }
                    if event.event_id() == event::PRESS && id_registered(widget, event::WIDGET_PRESS) {
                        if let Some(event_id) = widget.trigger_event(event::WIDGET_PRESS, event, &parent.layout, &mut self.solver) {
                            new_events.push((node_index, event_id));
                        }
                    }
                    if event.event_id() == event::MOUSE_SCROLL && id_registered(widget, event::MOUSE_SCROLL) {

                        // TODO trigger on mouse over parent, not child
                        widget.trigger_event(event::MOUSE_SCROLL, event, &parent.layout, &mut self.solver);
                    }
                }
            }
        }
        for (node_index, event_id) in new_events {
            let mut dfs = Dfs::new(&self.graph, self.root_index);
            while let Some(node_index) = dfs.next(&self.graph) {
                if let Some(parent_index) = self.parent_index(node_index) {
                    let (parent, widget) = self.graph.index_twice_mut(parent_index, node_index);
                    if id_registered(widget, event_id) {
                        widget.trigger_event(event_id, &Event::Update(UpdateArgs{dt:0.0}), &parent.layout, &mut self.solver);
                    }
                }
            }
        }
    }
}
