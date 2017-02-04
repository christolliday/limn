use std::any::Any;

use graphics::types::Color;
use petgraph::graph::NodeIndex;

use ui::{self, Ui};
use widget::{Drawable, Widget, WidgetStyle, EventHandler, StyleArgs, DrawArgs};
use widget::layout::WidgetLayout;
use resources::{resources, Id};
use util::{self, Point, Rectangle};

pub struct WidgetBuilder {
    pub id: Id,
    pub drawable: Option<Drawable>,
    pub layout: WidgetLayout,
    pub event_handlers: Vec<Box<EventHandler>>,
    pub debug_name: Option<String>,
    pub debug_color: Option<Color>,
    pub children: Vec<Box<WidgetBuilder>>,
}

impl WidgetBuilder {
    pub fn new() -> Self {
        WidgetBuilder {
            id: resources().widget_id(),
            drawable: None,
            layout: WidgetLayout::new(),
            event_handlers: Vec::new(),
            debug_name: None,
            debug_color: None,
            children: Vec::new(),
        }
    }
    pub fn set_drawable(mut self, drawable: Drawable) -> Self {
        self.drawable = Some(drawable);
        self
    }
    pub fn set_mouse_over_fn(mut self, mouse_over_fn: fn(Point, Rectangle) -> bool) -> Self {
        if let Some(ref mut drawable) = self.drawable {
            drawable.mouse_over_fn = Some(mouse_over_fn);
        }
        self
    }
    pub fn add_handler(mut self, handler: Box<EventHandler>) -> Self {
        self.event_handlers.push(handler);
        self
    }
    pub fn set_debug_name(mut self, name: &str) -> Self {
        self.debug_name = Some(name.to_owned());
        self
    }
    pub fn set_debug_color(mut self, color: Color) -> Self {
        self.debug_color = Some(color);
        self
    }
    // only method that is not chainable, because usually called out of order
    pub fn add_child(&mut self, mut widget: Box<WidgetBuilder>) {
        self.children.push(widget);
    }

    pub fn create(self, ui: &mut Ui, parent_index: Option<NodeIndex>) -> NodeIndex {
        let mut widget = Widget::new(self.id,
                                     self.drawable,
                                     self.layout,
                                     self.event_handlers,
                                     self.debug_name,
                                     self.debug_color);

        widget.layout.update_solver(&mut ui.solver);

        let widget_index = ui.add_widget(parent_index, widget);
        for child in self.children {
            child.create(ui, Some(widget_index));
        }
        widget_index
    }
}
