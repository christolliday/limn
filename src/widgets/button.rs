use glutin;

use text_layout::Align;

use event::{Target, WidgetEventArgs};
use widget::{WidgetBuilder, WidgetBuilderCore, BuildWidget};
use widget::property::{Property, PropChange};
use widget::property::states::*;
use layout::constraint::*;
use layout::{LayoutRef, LayoutVars};
use input::mouse::{WidgetMouseButton, ClickEvent};
use drawable::rect::{RectDrawable, RectStyleable};
use drawable::text::{TextDrawable, TextStyleable};
use util::{Scalar, Size, Color};
use color::*;

static COLOR_BUTTON_DEFAULT: Color = [0.8, 0.8, 0.8, 1.0];
static COLOR_BUTTON_PRESSED: Color = [0.6, 0.6, 0.6, 1.0];
static COLOR_BUTTON_ACTIVATED: Color = [0.4, 0.4, 0.4, 1.0];
static COLOR_BUTTON_ACTIVATED_PRESSED: Color = [0.3, 0.3, 0.3, 1.0];
static COLOR_BUTTON_INACTIVE: Color = [0.9, 0.9, 0.9, 1.0];
static COLOR_BUTTON_TEXT_INACTIVE: Color = [0.7, 0.7, 0.7, 1.0];

static BUTTON_BORDER: (Scalar, Color) = (1.0, [0.4, 0.4, 0.4, 1.0]);
static BUTTON_BORDER_INACTIVE: (Scalar, Color) = (1.0, [0.7, 0.7, 0.7, 1.0]);


lazy_static! {
    pub static ref STYLE_BUTTON: Vec<RectStyleable> = {
        style!(
            RectStyleable::BackgroundColor: selector!(COLOR_BUTTON_DEFAULT,
                ACTIVATED_PRESSED: COLOR_BUTTON_ACTIVATED_PRESSED,
                ACTIVATED: COLOR_BUTTON_ACTIVATED,
                PRESSED: COLOR_BUTTON_PRESSED,
                INACTIVE: COLOR_BUTTON_INACTIVE),
            RectStyleable::CornerRadius: Some(5.0),
            RectStyleable::Border: selector!(Some(BUTTON_BORDER),
                INACTIVE: Some(BUTTON_BORDER_INACTIVE))
        )
    };
    pub static ref STYLE_BUTTON_TEXT: Vec<TextStyleable> = {
        style!(TextStyleable::TextColor: selector!(BLACK, INACTIVE: COLOR_BUTTON_TEXT_INACTIVE))
    };
}

// show whether button is held down or not
fn button_handle_mouse_down(event: &WidgetMouseButton, args: WidgetEventArgs) {
    if !args.widget.props.contains(&Property::Inactive) {
        let &WidgetMouseButton(state, _) = event;
        let event = match state {
            glutin::ElementState::Pressed => PropChange::Add(Property::Pressed),
            glutin::ElementState::Released => PropChange::Remove(Property::Pressed),
        };
        event!(Target::SubTree(args.widget.id), event);
    }
}

pub enum ToggleEvent {
    On,
    Off,
}
// show whether toggle button is activated
fn toggle_button_handle_mouse(event: &WidgetMouseButton, args: WidgetEventArgs) {
    if let &WidgetMouseButton(glutin::ElementState::Released, _) = event {
        let (toggle_event, prop_event) = match args.widget.props.contains(&Property::Activated) {
            true => (ToggleEvent::Off, PropChange::Remove(Property::Activated)),
            false => (ToggleEvent::On, PropChange::Add(Property::Activated)),
        };
        event!(Target::Widget(args.widget.id), toggle_event);
        event!(Target::SubTree(args.widget.id), prop_event);
    }
}

pub struct ToggleButtonBuilder {
    pub widget: WidgetBuilder,
}
widget_builder!(ToggleButtonBuilder);

impl ToggleButtonBuilder {
    pub fn new() -> Self {
        let mut widget = WidgetBuilder::new();
        widget
            .set_drawable_with_style(RectDrawable::new(), STYLE_BUTTON.clone())
            .add_handler_fn(button_handle_mouse_down)
            .add_handler_fn(toggle_button_handle_mouse);
        layout!(widget: size(Size::new(70.0, 30.0)));

        ToggleButtonBuilder { widget: widget }
    }
    pub fn set_text(&mut self, on_text: &'static str, off_text: &'static str) -> &mut Self {

        let style = style!(parent: STYLE_BUTTON_TEXT,
            TextStyleable::Text: selector!(off_text.to_owned(),
                ACTIVATED: on_text.to_owned()),
            TextStyleable::Align: Align::Middle);
        let button_text_drawable = TextDrawable::default();
        let mut button_text_widget = WidgetBuilder::new();
        button_text_widget
            .set_drawable_with_style(button_text_drawable, style);
        layout!(button_text_widget: center(&self.widget));

        self.widget.add_child(button_text_widget);
        self
    }
    pub fn on_toggle<F>(&mut self, callback: F) -> &mut Self
        where F: Fn(&ToggleEvent, WidgetEventArgs) + 'static
    {
        self.widget.add_handler_fn(callback);
        self
    }
}

pub struct PushButtonBuilder {
    pub widget: WidgetBuilder,
}
widget_builder!(PushButtonBuilder);

impl PushButtonBuilder {
    pub fn new() -> Self {
        let mut widget = WidgetBuilder::new();
        widget
            .set_drawable_with_style(RectDrawable::new(), STYLE_BUTTON.clone())
            .add_handler_fn(button_handle_mouse_down);

        layout!(widget: size(Size::new(100.0, 50.0)));

        PushButtonBuilder { widget: widget }
    }
    pub fn set_text(&mut self, text: &'static str) -> &mut Self {

        let style = style!(parent: STYLE_BUTTON_TEXT,
            TextStyleable::Text: text.to_owned(),
            TextStyleable::Align: Align::Middle);

        let mut button_text_widget = WidgetBuilder::new();
        button_text_widget
            .set_drawable_with_style(TextDrawable::default(), style);
        layout!(button_text_widget: center(self.as_mut()));

        self.widget.add_child(button_text_widget);
        self
    }
}

pub trait WidgetClickable {
    fn on_click<F>(&mut self, on_click: F) -> &mut Self
        where F: Fn(&ClickEvent, &mut WidgetEventArgs) + 'static;
}
impl<B> WidgetClickable for B where B: AsMut<WidgetBuilder> {
    fn on_click<F>(&mut self, on_click: F) -> &mut Self
        where F: Fn(&ClickEvent, &mut WidgetEventArgs) + 'static
    {
        self.add_handler_fn(move |event, mut args| {
            (on_click)(event, &mut args);
            *args.handled = true;
        })
    }
}
