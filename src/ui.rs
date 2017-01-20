
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
use super::widget;
use widget::layout::WidgetLayout;
use widget::builder::WidgetBuilder;
use super::util::*;
use super::util;
use super::event;
use event::{Event, InputEvent};
use resources;
use resources::font::Font;
use backend::glyph::GlyphCache;
use backend::window::Window;

use resources::image::Texture;
use resources::Id;

use std::collections::HashMap;
use std::f64;
use std::cmp::max;

const DEBUG_BOUNDS: bool = false;

pub struct Resources {
    pub fonts: resources::Map<Font>,
    pub images: resources::Map<Texture>,
    pub next_widget_id: usize,
}
impl Resources {
    pub fn new() -> Self {
        let fonts = resources::Map::new();
        let images = resources::Map::new();
        Resources {
            fonts: fonts,
            images: images,
            next_widget_id: 0,
        }
    }
    pub fn widget_id(&mut self) -> Id {
        let id = self.next_widget_id;
        self.next_widget_id = id.wrapping_add(1);
        Id(id)
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
    pub root_index: Option<NodeIndex>,
    pub solver: Solver,
    pub input_state: InputState,
    pub widget_map: HashMap<Id, NodeIndex>,
}
impl Ui {
    pub fn new() -> Self {
        let mut solver = Solver::new();
        let mut graph = Graph::<Widget, ()>::new();
        let input_state = InputState::new();
        let mut ui = Ui {
            graph: graph,
            root_index: None,
            solver: solver,
            input_state: input_state,
            widget_map: HashMap::new(),
        };
        ui
    }
    pub fn set_root(&mut self, root_widget: WidgetBuilder) {
        self.root_index = Some(root_widget.create(self, None));
        let ref mut root = &mut self.graph[self.root_index.unwrap()];
        self.solver.add_edit_variable(root.layout.right, STRONG).unwrap();
        self.solver.add_edit_variable(root.layout.bottom, STRONG).unwrap();
        root.layout.top_left(Point { x: 0.0, y: 0.0 });
        root.layout.update_solver(&mut self.solver);
    }
    pub fn get_root(&mut self) -> &Widget {
        &self.graph[self.root_index.unwrap()]
    }
    pub fn get_root_dims(&mut self) -> Dimensions {
        let ref mut root = &mut self.graph[self.root_index.unwrap()];
        root.layout.get_dims(&mut self.solver)
    }
    pub fn window_resized(&mut self, window: &mut Window, window_dims: Dimensions) {
        let ref root = self.graph[self.root_index.unwrap()];
        self.solver.suggest_value(root.layout.right, window_dims.width).unwrap();
        self.solver.suggest_value(root.layout.bottom, window_dims.height).unwrap();
    }
    pub fn parents(&mut self, node_index: NodeIndex) -> Neighbors<()> {
        self.graph.neighbors_directed(node_index, Direction::Incoming)
    }
    pub fn children(&mut self, node_index: NodeIndex) -> Neighbors<()> {
        self.graph.neighbors_directed(node_index, Direction::Outgoing)
    }

    pub fn draw_node(&mut self, resources: &Resources, glyph_cache: &mut GlyphCache, context: Context, graphics: &mut G2d, node_index: NodeIndex, crop_to: Rectangle) {

        let crop_to = {
            let ref widget = self.graph[node_index];
            widget.draw(crop_to, resources, &mut self.solver, glyph_cache, context, graphics);

            util::crop_rect(crop_to, widget.layout.bounds(&mut self.solver))
        };

        let children: Vec<NodeIndex> = self.children(node_index).collect();
        for child_index in children {
            self.draw_node(resources, glyph_cache, context, graphics, child_index, crop_to);
        }
    }
    pub fn draw(&mut self, resources: &Resources, glyph_cache: &mut GlyphCache, context: Context, graphics: &mut G2d) {

        let index = self.root_index.unwrap().clone();
        self.draw_node(resources, glyph_cache, context, graphics, index, Rectangle { top: 0.0, left: 0.0, width: f64::MAX, height: f64::MAX });

        if DEBUG_BOUNDS {
            let mut dfs = Dfs::new(&self.graph, self.root_index.unwrap());
            while let Some(node_index) = dfs.next(&self.graph) {
                let ref widget = self.graph[node_index];
                draw_rect_outline(widget.layout.bounds(&mut self.solver), widget.debug_color, context, graphics);
            }
        }
    }
    pub fn add_widget(&mut self, parent_index: Option<NodeIndex>, child: Widget, id: Option<Id>) -> NodeIndex {
        let child_index = self.graph.add_node(child);
        if let Some(parent_index) = parent_index {
            self.graph.add_edge(parent_index, child_index, ());
        }
        if let Some(id) = id {
            self.widget_map.insert(id, child_index);
        }
        child_index
    }
    pub fn get_widget(&self, widget_id: Id) -> Option<&Widget> {
        self.widget_map.get(&widget_id).and_then(|node_index| {
            let ref widget = self.graph[NodeIndex::new(node_index.index())];
            return Some(widget);
        });
        None
    }
    pub fn send_event<E: Event>(&mut self, widget_id: Id, event: E) {

        let node_index = self.widget_map.get(&widget_id).unwrap();
        //and_then(|node_index| {
        let ref mut widget = self.graph[NodeIndex::new(node_index.index())];

        let fake = WidgetLayout::new();
        widget.trigger_event(event.event_id(), &event, &fake, &mut self.solver);
        /*
        let widget = self.get_widget(widget_id).unwrap();
        //let event = ClockEvent::new(CLOCK_TICK, 0);
        widget.trigger_event(event.event_id(), &event, &widget.layout, &mut self.solver);*/
    }
    pub fn handle_event(&mut self, event: input::Event) {
        if let Some(mouse) = event.mouse_cursor_args() {
            self.input_state.mouse = mouse.into();
        }
        let event = InputEvent::new(event.event_id(), event);
        self.post_event(event);
    }
    pub fn post_event<E: Event>(&mut self, event: E) {

        let mut new_events = Vec::new();
        let id_registered = |widget: &Widget, id| { widget.event_handlers.iter().any(|event_handler| event_handler.event_id() == id) };
        
        let mut dfs = Dfs::new(&self.graph, self.root_index.unwrap());
        while let Some(node_index) = dfs.next(&self.graph) {
            if let Some(parent_index) = self.parents(node_index).next() {
                let (parent, widget) = self.graph.index_twice_mut(parent_index, node_index);
                if widget.is_mouse_over(&mut self.solver, self.input_state.mouse) {
                    let widget_event_id = event::widget_event(&event);
                    if let Some(event_id) = widget_event_id {
                        if id_registered(widget, event_id) {
                            if let Some(event) = widget.trigger_event(event_id, &event, &parent.layout, &mut self.solver) {
                                new_events.push((node_index, event));
                            }
                        }
                    }
                }
            }
        }
        for (node_index, event) in new_events {
            let mut dfs = Dfs::new(&self.graph, self.root_index.unwrap());
            while let Some(node_index) = dfs.next(&self.graph) {
                if let Some(parent_index) = self.parents(node_index).next() {
                    let (parent, widget) = self.graph.index_twice_mut(parent_index, node_index);
                    let event_id = event.event_id();
                    if id_registered(widget, event_id) {
                        widget.trigger_event(event_id, event.as_ref(), &parent.layout, &mut self.solver);
                    }
                }
            }
        }
    }
}
