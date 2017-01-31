use std::any::Any;

use graphics::types::Color;
use petgraph::graph::NodeIndex;

use ui::{self, Ui};
use widget::{Widget, EventHandler, StyleArgs, DrawArgs, WidgetState};
use widget::layout::WidgetLayout;
use resources::{resources, Id};
use util::{self, Point, Rectangle};

pub struct WidgetBuilder {
    pub id: Id,
    pub draw_fn: Option<fn(DrawArgs)>,
    pub drawable: WidgetState,
    pub style: Option<Box<Any>>,
    pub style_fn: Option<fn(StyleArgs)>,
    pub mouse_over_fn: fn(Point, Rectangle) -> bool,
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
            draw_fn: None,
            drawable: WidgetState::new(),
            style: None,
            style_fn: None,
            mouse_over_fn: util::point_inside_rect,
            layout: WidgetLayout::new(),
            event_handlers: Vec::new(),
            debug_name: None,
            debug_color: None,
            children: Vec::new(),
        }
    }
    pub fn set_drawable(mut self, draw_fn: fn(DrawArgs), drawable: Box<Any>) -> Self {
        self.draw_fn = Some(draw_fn);
        self.drawable = WidgetState::new_state(drawable);
        self
    }
    pub fn set_style(mut self, style_fn: fn(StyleArgs), style: Box<Any>) -> Self {
        self.style_fn = Some(style_fn);
        self.style = Some(style);
        self
    }
    pub fn set_mouse_over_fn(mut self, mouse_over_fn: fn(Point, Rectangle) -> bool) -> Self {
        self.mouse_over_fn = mouse_over_fn;
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
        self.layout.add_child(&mut widget.layout);
        self.children.push(widget);
    }

    pub fn create(self,
                  ui: &mut Ui,
                  parent_index: Option<NodeIndex>)
                  -> NodeIndex {
        let mut widget = Widget::new(self.id, self.draw_fn, self.drawable, self.style, self.style_fn, self.mouse_over_fn, self.layout, self.event_handlers, self.debug_name, self.debug_color);

        widget.layout.update_solver(&mut ui.solver);

        let widget_index = ui.add_widget(parent_index, widget);
        for child in self.children {
            child.create(ui, Some(widget_index));
        }
        widget_index
    }
}
