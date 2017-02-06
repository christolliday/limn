use std::any::Any;

use graphics::types::Color;
use petgraph::graph::NodeIndex;
use cassowary::{self, Constraint};

use ui::{self, Ui};
use widget::{Drawable, Widget, WidgetStyle, EventHandler, StyleArgs, DrawArgs};
use widget::layout::{LayoutBuilder, WidgetConstraint};
use resources::{resources, Id};
use util::{self, Point, Rectangle};

pub struct WidgetBuilder {
    pub id: Id,
    pub drawable: Option<Drawable>,
    pub layout: LayoutBuilder,
    pub event_handlers: Vec<Box<EventHandler>>,
    pub debug_name: Option<String>,
    pub debug_color: Option<Color>,
    pub children: Vec<Box<WidgetBuilder>>,
    pub scrollable: bool,
}

impl WidgetBuilder {
    pub fn new() -> Self {
        WidgetBuilder {
            id: resources().widget_id(),
            drawable: None,
            layout: LayoutBuilder::new(),
            event_handlers: Vec::new(),
            debug_name: None,
            debug_color: None,
            children: Vec::new(),
            scrollable: false,
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
    pub fn set_scrollable(mut self) -> Self {
        self.scrollable = true;
        self
    }
    // only method that is not chainable, because usually called out of order
    pub fn add_child(&mut self, mut widget: Box<WidgetBuilder>) {
        if self.scrollable {
            widget.layout.scroll_inside(&self);
        } else {
            widget.layout.bound_by(&self, None);
        }
        self.children.push(widget);
    }

    pub fn build(self) -> (Vec<Box<WidgetBuilder>>, Vec<WidgetConstraint>, Widget) {

        if let Some(ref debug_name) = self.debug_name {
            cassowary::add_var_name(self.layout.vars.left, &format!("{}.left", debug_name));
            cassowary::add_var_name(self.layout.vars.top, &format!("{}.top", debug_name));
            cassowary::add_var_name(self.layout.vars.right, &format!("{}.right", debug_name));
            cassowary::add_var_name(self.layout.vars.bottom, &format!("{}.bottom", debug_name));
        }

        (
            self.children,
            self.layout.constraints,
            Widget::new(self.id,
                        self.drawable,
                        self.layout.vars,
                        self.event_handlers,
                        self.debug_name,
                        self.debug_color)
        )
    }
}
