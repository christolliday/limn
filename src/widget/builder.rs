use backend::gfx::G2d;
use graphics::Context;
use graphics::types::Color;

use input::EventId;
use util::*;
use util;
use widget::Widget;
use petgraph::graph::NodeIndex;

use ui::{Ui, Resources};
use widget::layout::WidgetLayout;
use widget::EventHandler;
use widget::DrawArgs;
use resources::Id;

use cassowary::Solver;
use cassowary::strength::*;

use std::any::Any;

pub struct WidgetBuilder {
    pub draw_fn: Option<fn(DrawArgs)>,
    pub drawable: Option<Box<Any>>,
    pub mouse_over_fn: fn(Point, Rectangle) -> bool,
    pub layout: WidgetLayout,
    pub event_handlers: Vec<Box<EventHandler>>,
    pub debug_color: Color,
    pub children: Vec<Box<WidgetBuilder>>,
    pub id: Option<Id>,
}

impl WidgetBuilder {
    pub fn new() -> Self {
        WidgetBuilder {
            draw_fn: None,
            drawable: None,
            mouse_over_fn: point_inside_rect,
            layout: WidgetLayout::new(),
            event_handlers: Vec::new(),
            debug_color: [0.0, 1.0, 0.0, 1.0],
            children: Vec::new(),
            id: None,
        }
    }
    pub fn set_drawable(&mut self, draw_fn: fn(DrawArgs), drawable: Box<Any>) {
        self.draw_fn = Some(draw_fn);
        self.drawable = Some(drawable);
    }
    pub fn set_mouse_over_fn(&mut self, mouse_over_fn: fn(Point, Rectangle) -> bool) {
        self.mouse_over_fn = mouse_over_fn;
    }
    pub fn debug_color(&mut self, color: Color) {
        self.debug_color = color;
    }
    pub fn add_child(&mut self, mut widget: Box<WidgetBuilder>) {
        self.layout.add_child(&mut widget.layout);
        self.children.push(widget);
    }
    pub fn set_id(&mut self, id: Id) {
        self.id = Some(id);
    }

    pub fn create(self,
                  ui: &mut Ui,
                  resources: &mut Resources,
                  parent_index: Option<NodeIndex>)
                  -> NodeIndex {
        let id = self.id.unwrap_or(resources.widget_id());
        let mut widget = Widget::new(id);

        if let (Some(draw_fn), Some(drawable)) = (self.draw_fn, self.drawable) {
            widget.set_drawable(draw_fn, drawable);
        }
        widget.set_mouse_over_fn(self.mouse_over_fn);
        widget.event_handlers = self.event_handlers;
        widget.layout = self.layout;
        widget.layout.update_solver(&mut ui.solver);

        let widget_index = ui.add_widget(parent_index, widget);
        for child in self.children {
            child.create(ui, resources, Some(widget_index));
        }
        widget_index
    }
}
