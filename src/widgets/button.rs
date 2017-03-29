use glutin;

use text_layout::Align;

use event::Target;
use widget::{WidgetBuilder, EventHandler, CallbackHandler, EventArgs};
use widget::style::{Value, Selector};
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
        let mut selector = Selector::new(COLOR_BUTTON_DEFAULT);
        selector.insert(&ACTIVATED_PRESSED, COLOR_BUTTON_ACTIVATED_PRESSED);
        selector.insert(&ACTIVATED, COLOR_BUTTON_ACTIVATED);
        selector.insert(&PRESSED, COLOR_BUTTON_PRESSED);
        selector.insert(&INACTIVE, COLOR_BUTTON_INACTIVE);

        vec!{ RectStyleField::BackgroundColor(Value::Selector(selector)), RectStyleField::CornerRadius(Value::Single(Some(8.0))) }
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
impl AsMut<WidgetBuilder> for ToggleButtonBuilder {
    fn as_mut(&mut self) -> &mut WidgetBuilder {
        &mut self.widget
    }
}
use widget::WidgetBuilderCore;
impl ToggleButtonBuilder {
    pub fn new() -> Self {

        let mut widget = WidgetBuilder::new();
        widget
            .set_drawable_with_style(RectDrawable::new(), STYLE_BUTTON.clone())
            .add_handler(ButtonDownHandler)
            .add_handler(ToggleEventHandler)
            .add_handler(PropChangeHandler);
        widget.layout().dimensions(Dimensions {
            width: 100.0,
            height: 50.0,
        });

        ToggleButtonBuilder { widget: widget }
    }
    pub fn set_text(&mut self, on_text: &'static str, off_text: &'static str) -> &mut Self {

        let mut selector = Selector::new(off_text.to_owned());
        selector.insert(&ACTIVATED, on_text.to_owned());
        let text_style = vec![TextStyleField::Text(Value::Selector(selector)),
                              TextStyleField::Align(Value::Single(Align::Middle))];

        let button_text_drawable = TextDrawable::default();
        let mut button_text_widget = WidgetBuilder::new();
        button_text_widget
            .set_drawable_with_style(button_text_drawable, text_style)
            .add_handler(PropChangeHandler);
        button_text_widget.layout().center(&self.widget.layout().vars);

        self.widget.add_child(button_text_widget);
        self
    }
    pub fn on_toggle<F>(&mut self, callback: F) -> &mut Self
        where F: Fn(&ToggleEvent, &mut EventArgs) + 'static {
        self.widget.add_handler(CallbackHandler::new(callback));
        self
    }
}

pub struct PushButtonBuilder {
    pub widget: WidgetBuilder,
}
impl AsMut<WidgetBuilder> for PushButtonBuilder {
    fn as_mut(&mut self) -> &mut WidgetBuilder {
        &mut self.widget
    }
}
impl PushButtonBuilder {
    pub fn new() -> Self {
        let mut widget = WidgetBuilder::new();
        widget
            .set_drawable_with_style(RectDrawable::new(), STYLE_BUTTON.clone())
            .add_handler(PropChangeHandler);

        widget.layout().dimensions(Dimensions {
            width: 100.0,
            height: 50.0,
        });

        PushButtonBuilder { widget: widget }
    }
    pub fn set_text(&mut self, text: &'static str) -> &mut Self {

        let text_style = vec![TextStyleField::Text(Value::Single(text.to_owned())),
                              TextStyleField::Align(Value::Single(Align::Middle))];
        let drawable = TextDrawable::default();
        let mut button_text_widget = WidgetBuilder::new();
        button_text_widget
            .set_drawable_with_style(drawable, text_style)
            .add_handler(PropChangeHandler);
        button_text_widget.layout().center(self.layout());

        self.widget.add_child(button_text_widget);
        self
    }
}

pub trait WidgetClickable {
    fn on_click<F>(&mut self, on_click: F) -> &mut Self
        where F: Fn(&ClickEvent, &mut EventArgs) + 'static;
}
impl<B> WidgetClickable for B where B: AsMut<WidgetBuilder> {
    fn on_click<F>(&mut self, on_click: F) -> &mut Self
        where F: Fn(&ClickEvent, &mut EventArgs) + 'static
    {
        self.add_handler(CallbackHandler::new(move |event, mut args| {
            (on_click)(event, &mut args);
            args.event_state.handled = true;
        }))
    }
}