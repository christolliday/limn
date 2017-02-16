use std::ops::Deref;

use glutin;
use linked_hash_map::LinkedHashMap;

use text_layout::Align;

use widget::{EventHandler, EventArgs};
use widget::property::{Property, PropsChangeEventHandler};
use widget::property::states::*;
use event::events::*;
use widgets::primitives::{self, RectStyleField};
use widgets::text::{self, TextStyleField};
use widget::builder::WidgetBuilder;
use widget::style::Value;
use util::{Dimensions, Color};
use color::*;

static COLOR_BUTTON_DEFAULT: Color = RED;
static COLOR_BUTTON_PRESSED: Color = [0.8, 0.0, 0.0, 1.0];
static COLOR_BUTTON_ACTIVATED: Color = WHITE;
static COLOR_BUTTON_ACTIVATED_PRESSED: Color = [0.9, 0.9, 0.9, 1.0];
static COLOR_BUTTON_INACTIVE: Color = [0.3, 0.3, 0.3, 1.0];

lazy_static! {
    pub static ref STYLE_BUTTON: Vec<RectStyleField> = {
        let mut selector = LinkedHashMap::new();
        selector.insert(STATE_ACTIVATED_PRESSED.deref().clone(), COLOR_BUTTON_ACTIVATED_PRESSED);
        selector.insert(STATE_ACTIVATED.deref().clone(), COLOR_BUTTON_ACTIVATED);
        selector.insert(STATE_PRESSED.deref().clone(), COLOR_BUTTON_PRESSED);
        selector.insert(STATE_INACTIVE.deref().clone(), COLOR_BUTTON_INACTIVE);
        selector.insert(STATE_DEFAULT.deref().clone(), COLOR_BUTTON_DEFAULT);

        vec!{ RectStyleField::BackgroundColor(Value::Selector((selector, COLOR_BUTTON_DEFAULT))), RectStyleField::CornerRadius(Value::Single(Some(8.0))) }
    };
}

// show whether button is held down or not
pub struct ButtonDownHandler {}
impl EventHandler<WidgetMouseButton> for ButtonDownHandler {
    fn handle(&mut self, event: &WidgetMouseButton, args: EventArgs) {
        match event.0 {
            glutin::Event::MouseInput(state, _) => {
                let pressed = match state {
                    glutin::ElementState::Pressed => true,
                    glutin::ElementState::Released => false,
                };
                args.event_queue.change_prop(args.widget_id, Property::Pressed, pressed);
            }
            _ => (),
        }
    }
}

// show whether toggle button is activated
pub struct ToggleEventHandler {}
impl EventHandler<WidgetMouseButton> for ToggleEventHandler {
    fn handle(&mut self, event: &WidgetMouseButton, args: EventArgs) {
        let EventArgs { widget_id, event_queue, .. } = args;
        match event.0 {
            glutin::Event::MouseInput(state, _) => {
                match state {
                    glutin::ElementState::Released => {
                        if let &mut Some(ref drawable) = args.drawable {
                            let activated = drawable.props.contains(&Property::Activated);
                            event_queue.change_prop(widget_id, Property::Activated, !activated);
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
            .set_drawable(primitives::rect_drawable(STYLE_BUTTON.clone()))
            .add_handler(ButtonDownHandler {})
            .add_handler(ToggleEventHandler {})
            .add_handler(PropsChangeEventHandler {});
        widget.layout.dimensions(Dimensions {
            width: 100.0,
            height: 50.0,
        });

        ToggleButtonBuilder { widget: widget }
    }
    pub fn set_text(mut self, on_text: &'static str, off_text: &'static str) -> Self {

        let mut selector = LinkedHashMap::new();
        selector.insert(STATE_ACTIVATED.deref().clone(), on_text.to_owned());
        let text_style = vec!{
            TextStyleField::Text(Value::Selector((selector, off_text.to_owned()))),
            TextStyleField::Align(Value::Single(Align::Middle)),
        };

        let button_text_drawable = text::text_drawable(text_style);
        let button_text_dims = text::measure(&button_text_drawable);
        let mut button_text_widget = WidgetBuilder::new()
            .set_drawable(button_text_drawable)
            .add_handler(PropsChangeEventHandler {});
        button_text_widget.layout.dimensions(button_text_dims);
        button_text_widget.layout.center(&self.widget);

        self.widget.add_child(button_text_widget);
        self
    }
}

pub struct ClickHandler<F> where F: Fn(&WidgetMouseButton, &mut EventArgs) {
    callback: F
}
impl<F: Fn(&WidgetMouseButton, &mut EventArgs)> ClickHandler<F> {
    pub fn new(callback: F) -> Self {
        ClickHandler { callback: callback }
    }
}
impl<F: Fn(&WidgetMouseButton, &mut EventArgs)> EventHandler<WidgetMouseButton> for ClickHandler<F> {
    fn handle(&mut self, event: &WidgetMouseButton, mut args: EventArgs) {
        match event.0 {
            glutin::Event::MouseInput(state, _) => {
                match state {
                    glutin::ElementState::Released => {
                        (self.callback)(event, &mut args);
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
            .set_drawable(primitives::rect_drawable(STYLE_BUTTON.clone()))
            .add_handler(PropsChangeEventHandler {});

        widget.layout.dimensions(Dimensions {
            width: 100.0,
            height: 50.0,
        });

        PushButtonBuilder { widget: widget }
    }
    pub fn set_text(mut self, text: &'static str) -> Self {

        let text_style = vec!{
            TextStyleField::Text(Value::Single(text.to_owned())),
            TextStyleField::Align(Value::Single(Align::Middle)),
        };
        let drawable = text::text_drawable(text_style);
        let button_text_dims = text::measure(&drawable);
        let mut button_text_widget = WidgetBuilder::new()
            .set_drawable(drawable)
            .add_handler(PropsChangeEventHandler {});
        button_text_widget.layout.dimensions(button_text_dims);
        button_text_widget.layout.center(&self.widget);

        self.widget.add_child(button_text_widget);
        self
    }
}
