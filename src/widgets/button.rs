use std::ops::Deref;

use glutin;
use linked_hash_map::LinkedHashMap;

use text_layout::Align;

use event::Target;
use widget::{WidgetBuilder, EventHandler, EventArgs};
use widget::style::Value;
use widget::property::{Property, PropChange, PropChangeHandler};
use widget::property::states::*;
use input::mouse::{ClickEvent, WidgetMouseButton};
use drawable::rect::{RectDrawable, RectStyleField};
use drawable::text::{TextDrawable, TextStyleField};
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
pub struct ButtonDownHandler;
impl EventHandler<WidgetMouseButton> for ButtonDownHandler {
    fn handle(&mut self, event: &WidgetMouseButton, args: EventArgs) {
        let &WidgetMouseButton(state, _) = event;
        let event = match state {
            glutin::ElementState::Pressed => PropChange::Add(Property::Pressed),
            glutin::ElementState::Released => PropChange::Remove(Property::Pressed),
        };
        args.queue.push(Target::SubTree(args.widget.id), event);
    }
}

pub enum ToggleEvent {
    On,
    Off,
}
// show whether toggle button is activated
pub struct ToggleEventHandler;
impl EventHandler<WidgetMouseButton> for ToggleEventHandler {
    fn handle(&mut self, event: &WidgetMouseButton, args: EventArgs) {
        if let &WidgetMouseButton(glutin::ElementState::Released, _) = event {
            let (toggle_event, prop_event) = match args.widget.props.contains(&Property::Activated) {
                true => (ToggleEvent::Off, PropChange::Remove(Property::Activated)),
                false => (ToggleEvent::On, PropChange::Add(Property::Activated)),
            };
            args.queue.push(Target::Widget(args.widget.id), toggle_event);
            args.queue.push(Target::SubTree(args.widget.id), prop_event);
        }
    }
}

pub struct ToggleButtonBuilder {
    pub widget: WidgetBuilder,
}
impl ToggleButtonBuilder {
    pub fn new() -> Self {

        let mut widget = WidgetBuilder::new()
            .set_drawable_with_style(RectDrawable::new(), STYLE_BUTTON.clone())
            .add_handler(ButtonDownHandler)
            .add_handler(ToggleEventHandler)
            .add_handler(PropChangeHandler);
        widget.layout.dimensions(Dimensions {
            width: 100.0,
            height: 50.0,
        });

        ToggleButtonBuilder { widget: widget }
    }
    pub fn set_text(mut self, on_text: &'static str, off_text: &'static str) -> Self {

        let mut selector = LinkedHashMap::new();
        selector.insert(STATE_ACTIVATED.deref().clone(), on_text.to_owned());
        let text_style = vec![TextStyleField::Text(Value::Selector((selector,
                                                                    off_text.to_owned()))),
                              TextStyleField::Align(Value::Single(Align::Middle))];

        let button_text_drawable = TextDrawable::new();
        let mut button_text_widget = WidgetBuilder::new()
            .set_drawable_with_style(button_text_drawable, text_style)
            .add_handler(PropChangeHandler);
        button_text_widget.layout.center(&self.widget);

        self.widget.add_child(button_text_widget);
        self
    }
}

pub struct ClickHandler<F>
    where F: Fn(&ClickEvent, &mut EventArgs)
{
    callback: F,
}
impl<F: Fn(&ClickEvent, &mut EventArgs)> ClickHandler<F> {
    pub fn new(callback: F) -> Self {
        ClickHandler { callback: callback }
    }
}
impl<F: Fn(&ClickEvent, &mut EventArgs)> EventHandler<ClickEvent>
    for ClickHandler<F> {
    fn handle(&mut self, event: &ClickEvent, mut args: EventArgs) {
        (self.callback)(event, &mut args);
        args.event_state.handled = true;
    }
}

pub struct PushButtonBuilder {
    pub widget: WidgetBuilder,
}
impl PushButtonBuilder {
    pub fn new() -> Self {
        let mut widget = WidgetBuilder::new()
            .set_drawable_with_style(RectDrawable::new(), STYLE_BUTTON.clone())
            .add_handler(PropChangeHandler);

        widget.layout.dimensions(Dimensions {
            width: 100.0,
            height: 50.0,
        });

        PushButtonBuilder { widget: widget }
    }
    pub fn set_text(mut self, text: &'static str) -> Self {

        let text_style = vec![TextStyleField::Text(Value::Single(text.to_owned())),
                              TextStyleField::Align(Value::Single(Align::Middle))];
        let drawable = TextDrawable::new();
        let mut button_text_widget = WidgetBuilder::new()
            .set_drawable_with_style(drawable, text_style)
            .add_handler(PropChangeHandler);
        button_text_widget.layout.center(&self.widget);

        self.widget.add_child(button_text_widget);
        self
    }
}

impl WidgetBuilder {
    pub fn on_click<F>(mut self, on_click: F) -> Self
        where F: Fn(&ClickEvent, &mut EventArgs) + 'static
    {
        self.controller.add_handler(ClickHandler::new(on_click));
        self
    }
}