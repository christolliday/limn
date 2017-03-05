use graphics::types::Color;
use cassowary;

use widget::{Drawable, Widget, WidgetContainer, EventHandler, HandlerWrapper, EventArgs};
use widget::layout::{LayoutBuilder, WidgetConstraint};
use widget::property::PropsChangeEventHandler;
use widgets::hover::HoverHandler;
use widgets::button::ClickHandler;
use widgets::scroll::{ScrollHandler, WidgetScrollHandler};
use widgets::drag::{DragWidgetPressHandler, DragMouseReleaseHandler, DragMouseCursorHandler,
                    DragInputHandler};
use resources::{resources, WidgetId};
use util::{Point, Rectangle};
use input::mouse::ClickEvent;

pub struct WidgetBuilder {
    pub id: WidgetId,
    pub drawable: Option<Drawable>,
    pub layout: LayoutBuilder,
    pub event_handlers: Vec<HandlerWrapper>,
    pub debug_name: Option<String>,
    pub debug_color: Option<Color>,
    pub children: Vec<WidgetBuilder>,
    pub contents_scroll: bool,
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
            contents_scroll: false,
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
    pub fn add_handler<E: 'static, T: EventHandler<E> + 'static>(mut self, handler: T) -> Self {
        self.event_handlers.push(HandlerWrapper::new(handler));
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
    // common handlers
    pub fn contents_scroll(mut self) -> Self {
        self.contents_scroll = true;
        self.add_handler(ScrollHandler)
    }
    pub fn on_click<F>(self, on_click: F) -> Self
        where F: Fn(&ClickEvent, &mut EventArgs) + 'static
    {
        self.add_handler(ClickHandler::new(on_click))
    }
    pub fn enable_hover(self) -> Self {
        self.add_handler(HoverHandler)
    }
    pub fn props_may_change(self) -> Self {
        self.add_handler(PropsChangeEventHandler)
    }
    pub fn scrollable(self) -> Self {
        self.add_handler(WidgetScrollHandler::new())
    }
    pub fn draggable(self) -> Self {
        self.add_handler(DragWidgetPressHandler)
            .add_handler(DragMouseCursorHandler)
            .add_handler(DragMouseReleaseHandler)
            .add_handler(DragInputHandler::new())
    }

    // only method that is not chainable, because usually called out of order
    pub fn add_child(&mut self, mut widget: WidgetBuilder) {
        if self.contents_scroll {
            widget.layout.scroll_inside(&self);
        } else {
            widget.layout.bound_by(&self, None);
        }
        self.children.push(widget);
    }

    pub fn build(self) -> (Vec<WidgetBuilder>, Vec<WidgetConstraint>, WidgetContainer) {

        if let Some(ref debug_name) = self.debug_name {
            cassowary::add_var_name(self.layout.vars.left, &format!("{}.left", debug_name));
            cassowary::add_var_name(self.layout.vars.top, &format!("{}.top", debug_name));
            cassowary::add_var_name(self.layout.vars.right, &format!("{}.right", debug_name));
            cassowary::add_var_name(self.layout.vars.bottom, &format!("{}.bottom", debug_name));
        }
        let widget = Widget::new(self.id,
                                 self.drawable,
                                 self.layout.vars,
                                 self.debug_name,
                                 self.debug_color);
        (self.children,
         self.layout.constraints,
         WidgetContainer {
             widget: widget,
             event_handlers: self.event_handlers,
         })
    }
}
