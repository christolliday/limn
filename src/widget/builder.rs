use std::any::Any;

use graphics::types::Color;
use petgraph::graph::NodeIndex;

use ui::{self, Ui};
use widget::{Widget, EventHandler, DrawArgs};
use widget::layout::WidgetLayout;
use resources::{resources, Id};
use util::{self, Point, Rectangle};

pub struct WidgetBuilder {
    pub draw_fn: Option<fn(DrawArgs)>,
    pub drawable: Option<Box<Any>>,
    pub mouse_over_fn: fn(Point, Rectangle) -> bool,
    pub layout: WidgetLayout,
    pub event_handlers: Vec<Box<EventHandler>>,
    pub debug_color: Color,
    pub children: Vec<Box<WidgetBuilder>>,
    pub id: Id,
    pub name: Option<String>,
}

impl WidgetBuilder {
    pub fn new() -> Self {
        WidgetBuilder {
            draw_fn: None,
            drawable: None,
            mouse_over_fn: util::point_inside_rect,
            layout: WidgetLayout::new(),
            event_handlers: Vec::new(),
            debug_color: [0.0, 1.0, 0.0, 1.0],
            children: Vec::new(),
            id: resources().widget_id(),
            name: None,
        }
    }
    pub fn set_name(&mut self, name: &str) {
        self.name = Some(name.to_owned());
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

    pub fn create(self,
                  ui: &mut Ui,
                  parent_index: Option<NodeIndex>)
                  -> NodeIndex {
        let mut widget = Widget::new(self.id);

        widget.drawable.state = self.drawable;
        widget.draw_fn = self.draw_fn;
        widget.name = self.name;
        widget.mouse_over_fn = self.mouse_over_fn;
        widget.event_handlers = self.event_handlers;
        widget.layout = self.layout;
        widget.layout.update_solver(&mut ui.solver);

        let widget_index = ui.add_widget(parent_index, widget);
        for child in self.children {
            child.create(ui, Some(widget_index));
        }
        widget_index
    }
}
