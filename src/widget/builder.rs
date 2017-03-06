use graphics::types::Color;
use cassowary;

use widget::{Widget, WidgetContainer, EventHandler, HandlerWrapper, EventArgs};
use widget::drawable::{Drawable, DrawableWrapper};
use widget::style::Style;
use widget::layout::{LayoutBuilder, WidgetConstraint};
use widget::property::{PropSet, Property, PropChangeHandler};
use widgets::hover::HoverHandler;
use widgets::button::ClickHandler;
use widgets::scroll::{ScrollHandler, WidgetScrollHandler};
use widgets::drag::DragWidgetPressHandler;
use resources::{resources, WidgetId};
use input::mouse::ClickEvent;

pub struct WidgetBuilder {
    pub id: WidgetId,
    pub drawable: Option<DrawableWrapper>,
    pub props: PropSet,
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
            props: PropSet::new(),
            layout: LayoutBuilder::new(),
            event_handlers: Vec::new(),
            debug_name: None,
            debug_color: None,
            children: Vec::new(),
            contents_scroll: false,
        }
    }
    pub fn set_drawable<T: Drawable + 'static>(mut self, drawable: T) -> Self {
        self.drawable = Some(DrawableWrapper::new(drawable));
        self
    }
    pub fn set_drawable_with_style<T: Drawable + 'static, S: Style<T> + 'static>(mut self, drawable: T, style: S) -> Self {
        self.drawable = Some(DrawableWrapper::new_with_style(drawable, style));
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
    pub fn set_inactive(mut self) -> Self {
        self.props.insert(Property::Inactive);
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
        self.add_handler(PropChangeHandler)
    }
    pub fn scrollable(self) -> Self {
        self.add_handler(WidgetScrollHandler::new())
    }
    pub fn draggable(self) -> Self {
        self.add_handler(DragWidgetPressHandler)
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
                                 self.props,
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
