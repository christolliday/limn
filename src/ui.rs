
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
use eventbus::{EventBus, EventAddress};
use super::util::*;
use super::util;
use super::event;
use event::{Event, InputEvent};
use resources;
use resources::font::Font;
use backend::glyph::GlyphCache;
use backend::window::Window;
use input::EventId;

use resources::image::Texture;
use resources::Id;

use std::collections::HashMap;
use std::f64;
use std::cmp::max;
use std::any::Any;

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
    pub event_bus: EventBus,
    pub event_queue: Vec<(EventAddress, Box<Event>)>,
}
impl Ui {
    pub fn new() -> Self {
        let mut solver = Solver::new();
        let mut graph = Graph::<Widget, ()>::new();
        let input_state = InputState::new();
        let mut event_bus = EventBus::new();
        let mut ui = Ui {
            graph: graph,
            root_index: None,
            solver: solver,
            input_state: input_state,
            widget_map: HashMap::new(),
            event_bus: event_bus,
            event_queue: Vec::new(),
        };
        ui
    }
    fn update_child(&mut self, widget_id: Id, event: &Event) {
        if self.widget_map.contains_key(&widget_id) {
            let node_index = *self.widget_map.get(&widget_id).unwrap();
            self.update_child_widget(node_index, event);
        } else {
            println!("widget id {:?} not in widget_map", widget_id);
        }
    }
    fn update_child_widget(&mut self, node_index: NodeIndex, event: &Event) {
        let children: Vec<NodeIndex> = self.children(node_index).collect();
        for child_index in children {
            let (parent, child) = self.graph.index_twice_mut(node_index, child_index);
            child.trigger_event(event.event_id(), event, &mut self.event_queue, &parent.layout, &mut self.solver);
        }
    }
    fn update_children(&mut self, widget_id: Id, event: &Event) {
        if self.widget_map.contains_key(&widget_id) {
            let node_index = *self.widget_map.get(&widget_id).unwrap();
            self.update_widget(node_index, event);
            self.update_children_widget(node_index, event);
        } else {
            println!("widget id {:?} not in widget_map", widget_id);
        }
    }
    fn update_children_widget(&mut self, node_index: NodeIndex, event: &Event) {
        let children: Vec<NodeIndex> = self.children(node_index).collect();
        for child_index in children {
            {
                let (parent, child) = self.graph.index_twice_mut(node_index, child_index);
                child.trigger_event(event.event_id(), event, &mut self.event_queue, &parent.layout, &mut self.solver);
            }
            self.update_children_widget(child_index, event);
        }
    }
    fn update(&mut self, widget_id: Id, event: &Event) {
        if self.widget_map.contains_key(&widget_id) {
            let node_index = *self.widget_map.get(&widget_id).unwrap();
            let ref mut widget = self.graph[node_index];
            let fake = WidgetLayout::new();
            widget.trigger_event(event.event_id(), event, &mut self.event_queue, &fake, &mut self.solver);
        } else {
            println!("widget id {:?} not in widget_map", widget_id);
        }
    }
    fn update_widget(&mut self, node_index: NodeIndex, event: &Event) {
        let ref mut widget = self.graph[node_index];
        let fake = WidgetLayout::new();
        widget.trigger_event(event.event_id(), event, &mut self.event_queue, &fake, &mut self.solver);
    }
    pub fn set_root(&mut self, root_widget: WidgetBuilder, resources: &mut Resources) {
        self.root_index = Some(root_widget.create(self, resources, None));
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
    pub fn add_widget(&mut self, parent_index: Option<NodeIndex>, child: Widget) -> NodeIndex {
        let id = child.id;
        let child_index = self.graph.add_node(child);
        if let Some(parent_index) = parent_index {
            self.graph.add_edge(parent_index, child_index, ());
        }
        self.widget_map.insert(id, child_index);
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
        let ref mut widget = self.graph[NodeIndex::new(node_index.index())];
        let fake = WidgetLayout::new();
        widget.trigger_event(event.event_id(), &event, &mut self.event_queue, &fake, &mut self.solver);
    }
    pub fn handle_event(&mut self, event: input::Event) {
        if let Some(mouse) = event.mouse_cursor_args() {
            self.input_state.mouse = mouse.into();
        }
        let event = InputEvent::new(event.event_id(), event);
        self.post_event(event);
    }
    pub fn post_event<E: Event>(&mut self, event: E) {
        let id_registered = |widget: &Widget, id| { widget.event_handlers.iter().any(|event_handler| event_handler.event_id() == id) };
        
        let mut dfs = Dfs::new(&self.graph, self.root_index.unwrap());
        while let Some(node_index) = dfs.next(&self.graph) {
            if let Some(parent_index) = self.parents(node_index).next() {
                let (parent, widget) = self.graph.index_twice_mut(parent_index, node_index);
                if widget.is_mouse_over(&mut self.solver, self.input_state.mouse) {
                    let widget_event_id = event::widget_event(&event);
                    if let Some(event_id) = widget_event_id {
                        if id_registered(widget, event_id) {
                            widget.trigger_event(event_id, &event, &mut self.event_queue, &parent.layout, &mut self.solver);
                        }
                    }
                }
            }
        }
        self.handle_event_queue();
    }
    pub fn handle_event_queue(&mut self) {
        while self.event_queue.len() > 0 {
            let (event_address, event) = self.event_queue.pop().unwrap();
            match event_address {
                EventAddress::IdAddress(address, id) => {
                    if address == "CHILD" {
                        self.update_child(Id(id), &*event);
                    } else if address == "CHILDREN" {
                        self.update_children(Id(id), &*event);
                    } else if address == "SELF" {
                        self.update(Id(id), &*event);
                    }
                }, _ => {}
            }
        }
    }
}
