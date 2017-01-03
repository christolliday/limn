use backend::gfx::G2d;
use graphics::Context;
use graphics::types::Color;

use event::Event;
use input::EventId;
use util::*;
use util;
use widget::Widget;
use petgraph::graph::NodeIndex;

use ui::{Ui, Resources};
use widget::layout::WidgetLayout;
use widget::EventHandler;

use cassowary::Solver;
use cassowary::strength::*;

use std::any::Any;

pub struct WidgetBuilder {
    pub draw_fn: Option<fn(&Any, Rectangle, Rectangle, &mut Resources, Context, &mut G2d)>,
    pub drawable: Option<Box<Any>>,
    pub mouse_over_fn: fn(Point, Rectangle) -> bool,
    pub layout: WidgetLayout,
    pub event_handlers: Vec<Box<EventHandler>>,
    pub debug_color: Color,
    pub children: Vec<Box<WidgetBuilder>>,
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
        }
    }
    pub fn set_drawable(&mut self, draw_fn: fn(&Any, Rectangle, Rectangle, &mut Resources, Context, &mut G2d), drawable: Box<Any>) {
        self.draw_fn = Some(draw_fn);
        self.drawable = Some(drawable);
    }
    pub fn set_mouse_over_fn(&mut self, mouse_over_fn: fn(Point, Rectangle) -> bool) {
        self.mouse_over_fn = mouse_over_fn;
    }
    pub fn debug_color(&mut self, color: Color) {
        self.debug_color = color;
    }
    pub fn create(self, ui: &mut Ui, parent_index: NodeIndex) -> NodeIndex {
        let mut widget = Widget::new();

        if let (Some(draw_fn), Some(drawable)) = (self.draw_fn, self.drawable) {
            widget.set_drawable(draw_fn, drawable);
        }
        widget.set_mouse_over_fn(self.mouse_over_fn);
        widget.event_handlers = self.event_handlers;
        widget.layout = self.layout;

        let widget_index = ui.add_widget(parent_index, widget);
        for child in self.children {
            let child_index = child.create(ui, widget_index);
        }
        widget_index
    }
}