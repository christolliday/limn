use std::collections::BTreeSet;
use std::ops::Deref;

use glutin;
use linked_hash_map::LinkedHashMap;
use graphics::types::Color;

use widget::{self, EventHandler, PropsChangeEventHandler, DrawableEventHandler, EventArgs, Property,
             PropSet};
use event::{self, EventId, EventAddress, WIDGET_PRESS, WIDGET_CHANGE_PROP};
use widgets::primitives::{self, RectStyle};
use widgets::text::{self, TextStyle, TextStyleField};
use widget::builder::WidgetBuilder;
use widget::style::Value;
use theme::{STATE_ACTIVATED};
use theme::{STYLE_TEXT, STYLE_TOGGLE_BUTTON};
use util::Dimensions;
use resources::Id;
use color::*;


// show whether button is held down or not
pub struct ButtonDownHandler {}
impl EventHandler for ButtonDownHandler {
    fn event_id(&self) -> EventId {
        WIDGET_PRESS
    }
    fn handle_event(&mut self, args: EventArgs) {
        let event = args.data.downcast_ref::<glutin::Event>().unwrap();
        match *event {
            glutin::Event::MouseInput(state, button) => {
                let pressed = match state {
                    glutin::ElementState::Pressed => true,
                    glutin::ElementState::Released => false,
                };
                args.event_queue.push(EventAddress::SubTree(args.widget_id),
                                      WIDGET_CHANGE_PROP,
                                      Box::new((Property::Pressed, pressed)));
            }
            _ => (),
        }
    }
}

// show whether toggle button is activated
pub struct ToggleEventHandler {}
impl EventHandler for ToggleEventHandler {
    fn event_id(&self) -> EventId {
        WIDGET_PRESS
    }
    fn handle_event(&mut self, args: EventArgs) {
        let EventArgs { widget_id, event_queue, .. } = args;
        let event = args.data.downcast_ref::<glutin::Event>().unwrap();
        match *event {
            glutin::Event::MouseInput(state, button) => {
                match state {
                    glutin::ElementState::Released => {
                        if let &mut Some(ref drawable) = args.drawable {
                            let activated = drawable.props.contains(&Property::Activated);
                            event_queue.push(EventAddress::SubTree(widget_id),
                                             WIDGET_CHANGE_PROP,
                                             Box::new((Property::Activated, !activated)));
                        }
                    }
                    _ => (),
                }
            }
            _ => (),
        }
    }
}

pub struct ToggleButtonBuilder {
    pub widget: WidgetBuilder,
}
impl ToggleButtonBuilder {
    pub fn new() -> Self {

        let mut widget = WidgetBuilder::new()
            .set_drawable(primitives::rect_drawable(STYLE_TOGGLE_BUTTON.clone()))
            .add_handler(Box::new(ButtonDownHandler {}))
            .add_handler(Box::new(ToggleEventHandler {}))
            .add_handler(Box::new(PropsChangeEventHandler {}));
        widget.layout.dimensions(Dimensions {
            width: 100.0,
            height: 50.0,
        });

        ToggleButtonBuilder { widget: widget }
    }
    pub fn set_text(mut self, on_text: &'static str, off_text: &'static str) -> Self {

        let mut selector = LinkedHashMap::new();
        selector.insert(STATE_ACTIVATED.deref().clone(), on_text.to_owned());
        let text_style_value = Value::Selector((selector, off_text.to_owned()));
        let mut text_style = STYLE_TEXT.clone();
        text_style.text = text_style_value;

        let button_text_drawable = text::text_drawable(text_style);
        let button_text_dims = text::measure_dims_no_wrap(&button_text_drawable);
        let mut button_text_widget = WidgetBuilder::new()
            .set_drawable(button_text_drawable)
            .add_handler(Box::new(PropsChangeEventHandler {}));
        button_text_widget.layout.dimensions(button_text_dims);
        button_text_widget.layout.center(&self.widget);

        self.widget.add_child(Box::new(button_text_widget));
        self
    }
}

struct ClickHandler<F> where F: Fn(&mut EventArgs) {
    callback: F
}
impl<F> EventHandler for ClickHandler<F> where F: Fn(&mut EventArgs) {
    fn event_id(&self) -> EventId {
        WIDGET_PRESS
    }
    fn handle_event(&mut self, mut args: EventArgs) {
        let event = args.data.downcast_ref::<glutin::Event>().unwrap();
        match *event {
            glutin::Event::MouseInput(state, button) => {
                match state {
                    glutin::ElementState::Released => {
                        (self.callback)(&mut args);
                        args.event_state.handled = true;
                    }, _ => ()
                }
            }, _ => ()
        }
    }
}

pub struct PushButtonBuilder {
    pub widget: WidgetBuilder,
}
impl PushButtonBuilder {
    pub fn new() -> Self {
        let mut widget = WidgetBuilder::new()
            .set_drawable(primitives::rect_drawable(STYLE_TOGGLE_BUTTON.clone()));

        widget.layout.dimensions(Dimensions {
            width: 100.0,
            height: 50.0,
        });

        PushButtonBuilder { widget: widget }
    }
    pub fn set_text(mut self, text: &'static str) -> Self {

        let text_fields = vec!{ TextStyleField::text(Value::Single(text.to_owned())) };
        let text_style = TextStyle::from(text_fields);
        let drawable = text::text_drawable(text_style);
        let button_text_dims = text::measure_dims_no_wrap(&drawable);
        let mut button_text_widget = WidgetBuilder::new().set_drawable(drawable);
        button_text_widget.layout.dimensions(button_text_dims);
        button_text_widget.layout.center(&self.widget);

        self.widget.add_child(Box::new(button_text_widget));
        self
    }
    pub fn set_on_click<F>(mut self, on_click: F) -> Self
        where F: Fn(&mut EventArgs) + 'static {
        self.widget.event_handlers.push(Box::new(ClickHandler{ callback: on_click }));
        self
    }
}
