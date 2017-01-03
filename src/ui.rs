
use backend::gfx::G2d;

use petgraph::Graph;
use petgraph::graph::NodeIndex;
use petgraph::visit::Dfs;
use petgraph::Direction;
use petgraph::graph::Neighbors;

use input;
use input::{GenericEvent, MouseCursorEvent, UpdateArgs};

use cassowary::{Solver, Constraint};
use cassowary::WeightedRelation::*;
use cassowary::strength::*;

use graphics::Context;
use super::widget::*;
use super::widget::primitives::EmptyDrawable;
use super::widget;
use super::util::*;
use super::util;
use super::event;
use event::Event;
use resources;
use resources::font::Font;
use backend::glyph::GlyphCache;
use backend::window::Window;

use resources::image::Texture;

use std::f64;

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
        solver.add_constraints(&constraints);
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
    pub fn parents(&mut self, node_index: NodeIndex) -> Neighbors<()> {
        self.graph.neighbors_directed(node_index, Direction::Incoming)
    }
    pub fn children(&mut self, node_index: NodeIndex) -> Neighbors<()> {
        self.graph.neighbors_directed(node_index, Direction::Outgoing)
    }

    pub fn draw_node(&mut self, c: Context, g: &mut G2d, node_index: NodeIndex, crop_to: Rectangle) {

        let crop_to = {
            let ref widget = self.graph[node_index];
            widget.draw(crop_to, &mut self.resources, &mut self.solver, c, g);

            util::crop_rect(crop_to, widget.layout.bounds(&mut self.solver))
        };

        let children: Vec<NodeIndex> = self.children(node_index).collect();
        for child_index in children {
            self.draw_node(c, g, child_index, crop_to);
        }
    }
    pub fn draw(&mut self, c: Context, g: &mut G2d) {

        let index = self.root_index.clone();
        self.draw_node(c, g, index, Rectangle { top: 0.0, left: 0.0, width: f64::MAX, height: f64::MAX });

        if DEBUG_BOUNDS {
            let mut dfs = Dfs::new(&self.graph, self.root_index);
            while let Some(node_index) = dfs.next(&self.graph) {
                let ref widget = self.graph[node_index];
                draw_rect_outline(widget.layout.bounds(&mut self.solver),
                                  widget.debug_color,
                                  c,
                                  g);
            }
        }
    }
    pub fn add_widget(&mut self, parent_index: NodeIndex, child: Widget) -> NodeIndex {
        let child_index = self.graph.add_node(child);
        self.graph.add_edge(parent_index, child_index, ());

        let (parent, child) = self.graph.index_twice_mut(parent_index, child_index);

        parent.layout.add_child(&mut child.layout, &mut self.solver);

        child_index
    }
    pub fn handle_event(&mut self, event: input::Event) {
        if let Some(mouse) = event.mouse_cursor_args() {
            self.input_state.mouse = mouse.into();
        }
        self.post_event(event);
    }
    pub fn post_event(&mut self, event: input::Event) {
        let event = Event::Input(event);

        let mut new_events = Vec::new();
        let id_registered = |widget: &Widget, id| { widget.event_handlers.iter().any(|event_handler| event_handler.event_id() == id) };
        
        let mut dfs = Dfs::new(&self.graph, self.root_index);
        while let Some(node_index) = dfs.next(&self.graph) {
            if let Some(parent_index) = self.parents(node_index).next() {
                let (parent, widget) = self.graph.index_twice_mut(parent_index, node_index);
                if widget.is_mouse_over(&mut self.solver, self.input_state.mouse) {
                    if let Some(widget_event) = event::widget_event(&event) {
                        if id_registered(widget, widget_event) {
                            if let Some(event) = widget.trigger_event(widget_event, event.clone(), &parent.layout, &mut self.solver) {
                                new_events.push((node_index, event));
                            }
                        }
                    }
                }
            }
        }
        for (node_index, event) in new_events {
            let mut dfs = Dfs::new(&self.graph, self.root_index);
            while let Some(node_index) = dfs.next(&self.graph) {
                if let Some(parent_index) = self.parents(node_index).next() {
                    let (parent, widget) = self.graph.index_twice_mut(parent_index, node_index);
                    let event_id = event.event_id();
                    if id_registered(widget, event_id) {
                        widget.trigger_event(event_id, event.clone(), &parent.layout, &mut self.solver);
                    }
                }
            }
        }
    }
}
