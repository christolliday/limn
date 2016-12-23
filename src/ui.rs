
use backend::gfx::G2d;

use petgraph::Graph;
use petgraph::graph::NodeIndex;
use petgraph::visit::Dfs;

use input::{Event, Input, Motion};

use cassowary::{Solver, Variable, Constraint};
use cassowary::strength::*;

use graphics::Context;
use super::widget::*;
use super::util::*;
use resources;
use resources::font::Font;
use backend::glyph::GlyphCache;
use backend::window::Window;
use gfx_device_gl::Factory;
use gfx_graphics::{TextureSettings, Flip};

use resources::{Map,Id};
use resources::image::Texture;
use std::path::Path;

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

pub struct Ui {
    graph: Graph<Widget, ()>,
    pub root: NodeIndex,
    constraints: Vec<Constraint>,
    pub solver: Solver,
    pub window_width: Variable,
    pub window_height: Variable,
    pub resources: Resources,
}
impl Ui {
    pub fn new(window: &mut Window, window_dim: Dimensions) -> Self {
        let root = Widget::new(Box::new(EmptyDrawable{}));
        let window_width = Variable::new();
        let window_height = Variable::new();
        let constraints = Vec::new();
        let mut solver = Solver::new();
        solver.add_edit_variable(window_width, STRONG).unwrap();
        solver.add_edit_variable(window_height, STRONG).unwrap();
        solver.suggest_value(window_width, window_dim.width).unwrap();
        solver.suggest_value(window_height, window_dim.height).unwrap();

        let mut graph = Graph::<Widget, ()>::new();
        let root = graph.add_node(root);

        let glyph_cache = GlyphCache::new(&mut window.context.factory,
                                          window_dim.width as u32,
                                          window_dim.height as u32);
        
        let resources = Resources::new(glyph_cache);
        Ui {
            graph: graph,
            root: root,
            solver: solver,
            constraints: constraints,
            window_width: window_width,
            window_height: window_height,
            resources: resources,
        }
    }
    pub fn resize_window(&mut self, window_dims: [u32; 2]) {
        self.solver.suggest_value(self.window_width, window_dims[0] as f64).unwrap();
        self.solver.suggest_value(self.window_height, window_dims[1] as f64).unwrap();
    }
    pub fn init(&mut self) {
        let mut dfs = Dfs::new(&self.graph, self.root);
        while let Some(node_index) = dfs.next(&self.graph) {
            let ref mut node = self.graph[node_index];
            let constraints = &mut node.layout.constraints;
            self.constraints.append(constraints);
        }
        self.solver.add_constraints(&self.constraints).unwrap();
    }
    pub fn draw(&mut self, c: Context, g: &mut G2d) {
        let mut dfs = Dfs::new(&self.graph, self.root);
        while let Some(node_index) = dfs.next(&self.graph) {
            let ref widget = self.graph[node_index];
            widget.draw(&mut self.resources, &mut self.solver, c, g);
        }
    }
    pub fn add_widget(&mut self, parent_index: NodeIndex, child: Widget) -> NodeIndex {
        let child_index = self.graph.add_node(child);
        self.graph.add_edge(parent_index, child_index, ());

        let (parent, child) = self.graph.index_twice_mut(parent_index, child_index);
        child.layout.bound_by(&parent.layout);

        child_index
    }
    pub fn post_event(&mut self, event: &Event) {
        let mut dfs = Dfs::new(&self.graph, self.root);
        while let Some(node_index) = dfs.next(&self.graph) {
            let ref widget = self.graph[node_index];
            match event {
                &Event::Input(Input::Move(Motion::MouseCursor(x, y))) => {
                    let pos = Point { x: x, y: y };
                    for listener in &widget.listeners {
                        if widget.is_mouse_over(&mut self.solver, pos) && listener.matches(event) {
                            listener.handle_event(event);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
