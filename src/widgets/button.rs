use glutin;

use text_layout::Align;
use cassowary::strength::*;

use layout::constraint::*;
use event::WidgetEventArgs;
use widget::WidgetBuilder;
use widget::property::{Property, PropChange};
use widget::property::states::*;
use widgets::text::TextBuilder;
use input::mouse::{WidgetMouseButton, ClickEvent};
use drawable::rect::{RectDrawable, RectStyleable};
use drawable::text::TextStyleable;
use util::Size;
use color::*;

static COLOR_BUTTON_DEFAULT: Color = GRAY_80;
static COLOR_BUTTON_PRESSED: Color = GRAY_60;
static COLOR_BUTTON_ACTIVATED: Color = GRAY_40;
static COLOR_BUTTON_ACTIVATED_PRESSED: Color = GRAY_30;
static COLOR_BUTTON_INACTIVE: Color = GRAY_90;
static COLOR_BUTTON_TEXT_INACTIVE: Color = GRAY_70;

static BUTTON_BORDER: (f32, Color) = (1.0, GRAY_40);
static BUTTON_BORDER_INACTIVE: (f32, Color) = (1.0, GRAY_70);


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
fn button_handle_mouse_down(event: &WidgetMouseButton, mut args: WidgetEventArgs) {
    if !args.widget.props().contains(&Property::Inactive) {
        let &WidgetMouseButton(state, _) = event;
        let event = match state {
            glutin::ElementState::Pressed => PropChange::Add(Property::Pressed),
            glutin::ElementState::Released => PropChange::Remove(Property::Pressed),
        };
        args.widget.event_subtree(event);
    }
}

pub enum ToggleEvent {
    On,
    Off,
}
// show whether toggle button is activated
fn toggle_button_handle_mouse(event: &WidgetMouseButton, mut args: WidgetEventArgs) {
    if let WidgetMouseButton(glutin::ElementState::Released, _) = *event {
        let (toggle_event, prop_event) = match args.widget.props().contains(&Property::Activated) {
            true => (ToggleEvent::Off, PropChange::Remove(Property::Activated)),
            false => (ToggleEvent::On, PropChange::Add(Property::Activated)),
        };
        args.widget.event(toggle_event);
        args.widget.event_subtree(prop_event);
    }
}

pub struct ToggleButtonBuilder {
    pub widget: WidgetBuilder,
}
widget_wrapper!(ToggleButtonBuilder);

impl ToggleButtonBuilder {
    pub fn new() -> Self {
        let mut widget = WidgetBuilder::new("toggle_button");
        widget
            .set_drawable_with_style(RectDrawable::new(), STYLE_BUTTON.clone())
            .add_handler_fn(button_handle_mouse_down)
            .add_handler_fn(toggle_button_handle_mouse);
        widget.layout().add(constraints![
            min_size(Size::new(70.0, 30.0)),
            shrink(),
        ]);

        ToggleButtonBuilder { widget: widget }
    }
    pub fn set_text(&mut self, on_text: &'static str, off_text: &'static str) -> &mut Self {

        let style = style!(parent: STYLE_BUTTON_TEXT,
            TextStyleable::Text: selector!(off_text.to_owned(),
                ACTIVATED: on_text.to_owned()),
            TextStyleable::Align: Align::Middle);
        let mut button_text_widget = TextBuilder::new_with_style(style);
        button_text_widget.set_name("button_text");
        button_text_widget.layout().add(constraints![
            bound_left(&self.widget).padding(20.0),
            bound_right(&self.widget).padding(20.0),
            bound_top(&self.widget).padding(10.0),
            bound_bottom(&self.widget).padding(10.0),
            center(&self.widget),
        ]);

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
widget_wrapper!(PushButtonBuilder);

impl PushButtonBuilder {
    pub fn new() -> Self {
        let mut widget = WidgetBuilder::new("push_button");
        widget
            .set_drawable_with_style(RectDrawable::new(), STYLE_BUTTON.clone())
            .add_handler_fn(button_handle_mouse_down);

        widget.layout().add(constraints![
            min_size(Size::new(100.0, 50.0)).strength(STRONG),
            shrink(),
        ]);

        PushButtonBuilder { widget: widget }
    }
    pub fn set_text(&mut self, text: &'static str) -> &mut Self {

        let style = style!(parent: STYLE_BUTTON_TEXT,
            TextStyleable::Text: text.to_owned(),
            TextStyleable::Align: Align::Middle);

        let mut button_text_widget = TextBuilder::new_with_style(style);
        button_text_widget.set_name("button_text");
        button_text_widget.layout().add(constraints![
            bound_left(&self.widget).padding(20.0),
            bound_right(&self.widget).padding(20.0),
            bound_top(&self.widget).padding(10.0),
            bound_bottom(&self.widget).padding(10.0),
            center(&self.widget),
        ]);

        self.widget.add_child(button_text_widget);
        self
    }
}

impl WidgetBuilder {
    pub fn on_click<F>(&mut self, on_click: F) -> &mut Self
        where F: Fn(&ClickEvent, &mut WidgetEventArgs) + 'static
    {
        self.add_handler_fn(move |event, mut args| {
            (on_click)(event, &mut args);
            *args.handled = true;
        })
    }
}
