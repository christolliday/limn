use std::collections::BTreeSet;
use std::ops::Deref;

use glutin;
use linked_hash_map::LinkedHashMap;
use graphics::types::Color;

use widget::{self, EventHandler, PropsChangeEventHandler, DrawableEventHandler, EventArgs, Property, PropSet};
use event::{self, EventId, EventAddress, WIDGET_CHANGE_PROP};
use widgets::primitives::{self, RectDrawable, RectStyle};
use widgets::text::{self, TextDrawable, TextStyle, TEXT_STYLE_DEFAULT};
use widget::builder::WidgetBuilder;
use widget::style::StyleSheet;
use util::Dimensions;
use resources::Id;
use color::*;


static COLOR_BUTTON_DEFAULT: Color = RED;
static COLOR_BUTTON_PRESSED: Color = [0.8, 0.0, 0.0, 1.0];
static COLOR_BUTTON_ACTIVATED: Color = WHITE;
static COLOR_BUTTON_ACTIVATED_PRESSED: Color = [0.9, 0.9, 0.9, 1.0];

lazy_static! {
    pub static ref STATE_DEFAULT: PropSet = btreeset!{};
    pub static ref STATE_PRESSED: PropSet = btreeset!{Property::Pressed};
    pub static ref STATE_ACTIVATED: PropSet = btreeset!{Property::Activated};
    pub static ref STATE_ACTIVATED_PRESSED: PropSet = btreeset!{Property::Activated, Property::Pressed};
    pub static ref TOGGLE_RECT_STYLE: RectStyle = {
        let mut style = LinkedHashMap::new();
        style.insert(STATE_ACTIVATED_PRESSED.deref().clone(), COLOR_BUTTON_ACTIVATED_PRESSED);
        style.insert(STATE_ACTIVATED.deref().clone(), COLOR_BUTTON_ACTIVATED);
        style.insert(STATE_PRESSED.deref().clone(), COLOR_BUTTON_PRESSED);
        RectStyle { background: StyleSheet::new(style, COLOR_BUTTON_DEFAULT) }
    };
}

// show whether button is held down or not
pub struct ButtonDownHandler {}
impl EventHandler for ButtonDownHandler {
    fn event_id(&self) -> EventId {
        event::WIDGET_PRESS
    }
    fn handle_event(&mut self, args: EventArgs) {
        let event = args.data.downcast_ref::<glutin::Event>().unwrap();
        match *event {
            glutin::Event::MouseInput(state, button) => {
                let pressed = match state {
                    glutin::ElementState::Pressed => true,
                    glutin::ElementState::Released => false,
                };
                args.event_queue.push(EventAddress::SubTree(args.widget_id), WIDGET_CHANGE_PROP, Box::new((Property::Pressed, pressed)));
            }, _ => ()
        }
    }
}

// show whether toggle button is activated
pub struct ToggleEventHandler {}
impl EventHandler for ToggleEventHandler {
    fn event_id(&self) -> EventId {
        event::WIDGET_PRESS
    }
    fn handle_event(&mut self, args: EventArgs) {
        let EventArgs { props, widget_id, event_queue, .. } = args;
        let event = args.data.downcast_ref::<glutin::Event>().unwrap();
        match *event {
            glutin::Event::MouseInput(state, button) => {
                match state {
                    glutin::ElementState::Released => {
                        let activated = props.contains(&Property::Activated);
                        event_queue.push(EventAddress::SubTree(widget_id), WIDGET_CHANGE_PROP, Box::new((Property::Activated, !activated)));
                    }, _ => ()
                }
            }, _ => ()
        }
    }
}

pub struct ToggleButtonBuilder {
    pub widget: WidgetBuilder,
}
impl ToggleButtonBuilder {
    pub fn new() -> Self {
        let rect = RectDrawable::new(&TOGGLE_RECT_STYLE);

        let mut widget = WidgetBuilder::new()
            .set_drawable(primitives::draw_rect, Box::new(rect))
            .set_style(primitives::apply_rect_style, Box::new(TOGGLE_RECT_STYLE.clone()))
            .add_handler(Box::new(ButtonDownHandler{}))
            .add_handler(Box::new(ToggleEventHandler{}))
            .add_handler(Box::new(PropsChangeEventHandler{}));
        widget.layout.dimensions(Dimensions {
            width: 100.0,
            height: 50.0,
        });

        ToggleButtonBuilder { widget: widget }
    }
    pub fn set_text(mut self,
                    on_text: &'static str,
                    off_text: &'static str,
                    font_id: Id) -> Self {

        let mut style = LinkedHashMap::new();
        style.insert(STATE_ACTIVATED.deref().clone(), on_text.to_owned());
        let text_style = StyleSheet::new(style, off_text.to_owned());
        let mut text_style_set = TEXT_STYLE_DEFAULT.clone();
        text_style_set.text = text_style;

        let button_text_drawable = TextDrawable::new_style(&text_style_set);
        let button_text_dims = button_text_drawable.measure_dims_no_wrap();
        let mut button_text_widget = WidgetBuilder::new()
            .set_drawable(text::draw_text, Box::new(button_text_drawable))
            .set_style(text::apply_text_style, Box::new(text_style_set))
            .add_handler(Box::new(PropsChangeEventHandler{}));
        button_text_widget.layout.dimensions(button_text_dims);
        button_text_widget.layout.center(&self.widget.layout);

        self.widget.add_child(Box::new(button_text_widget));
        self
    }
}

pub struct PushButtonBuilder {
    pub widget: WidgetBuilder,
}
impl PushButtonBuilder {
    pub fn new() -> Self {
        let rect = RectDrawable { background: RED };
        let mut widget = WidgetBuilder::new()
            .set_drawable(primitives::draw_rect, Box::new(rect));

        widget.layout.dimensions(Dimensions {
            width: 100.0,
            height: 50.0,
        });

        PushButtonBuilder { widget: widget }
    }
    pub fn set_text(mut self, text: &'static str, font_id: Id) -> Self {

        let button_text_drawable = TextDrawable::new_style(TEXT_STYLE_DEFAULT.clone().with_text(text));
        let button_text_dims = button_text_drawable.measure_dims_no_wrap();
        let mut button_text_widget = WidgetBuilder::new()
            .set_drawable(text::draw_text, Box::new(button_text_drawable));
        button_text_widget.layout.dimensions(button_text_dims);
        button_text_widget.layout.center(&self.widget.layout);

        self.widget.add_child(Box::new(button_text_widget));
        self
    }
}
