
use backend::gfx::G2d;

use petgraph::Graph;
use petgraph::graph::NodeIndex;
use petgraph::visit::Dfs;

use input::{Event, Input, Motion};

use cassowary::{Solver, Variable, Constraint};
use cassowary::WeightedRelation::*;
use cassowary::strength::*;

use graphics::Context;
use super::widget::*;
use super::widget::primitives::EmptyDrawable;
use super::widget;
use super::util::*;
use resources;
use resources::font::Font;
use backend::glyph::GlyphCache;
use backend::window::Window;
use gfx_device_gl::Factory;
use gfx_graphics::{TextureSettings, Flip};

use resources::{Map, Id};
use resources::image::Texture;
use std::path::Path;
use std::any::Any;

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

pub struct Ui {
    pub graph: Graph<Widget, ()>,
    pub root_index: NodeIndex,
    constraints: Vec<Constraint>,
    pub solver: Solver,
    pub resources: Resources,
}
impl Ui {
    pub fn new(window: &mut Window, window_dims: Dimensions) -> Self {
        let root = Widget::new(widget::primitives::draw_nothing, Box::new(EmptyDrawable {}));
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
        let mut ui = Ui {
            graph: graph,
            root_index: root_index,
            solver: solver,
            constraints: constraints,
            resources: resources,
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
        let mut dfs = Dfs::new(&self.graph, self.root_index);
        while let Some(node_index) = dfs.next(&self.graph) {
            let ref mut node = self.graph[node_index];
            let constraints = &mut node.layout.constraints;
            self.constraints.append(constraints);
        }
        self.solver.add_constraints(&self.constraints).unwrap();
    }
    pub fn draw(&mut self, c: Context, g: &mut G2d) {
        let mut dfs = Dfs::new(&self.graph, self.root_index);
        while let Some(node_index) = dfs.next(&self.graph) {
            let ref widget = self.graph[node_index];
            if DEBUG_BOUNDS {
                draw_rect_outline(widget.layout.bounds(&mut self.solver),
                                  [0.0, 1.0, 1.0, 1.0],
                                  c,
                                  g);
            }
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
        let mut dfs = Dfs::new(&self.graph, self.root_index);
        while let Some(node_index) = dfs.next(&self.graph) {
            let ref mut widget = self.graph[node_index];
            widget.handle_event(&mut self.solver, event);
        }
    }
}
