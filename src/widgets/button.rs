use glutin;

use text_layout::Align;
use cassowary::strength::*;

use layout::constraint::*;
use event::EventArgs;
use widget::WidgetBuilder;
use widget::property::Property;
use widget::property::states::*;
use widgets::text::TextBuilder;
use input::mouse::{WidgetMouseButton, ClickEvent};
use draw::rect::{RectState, RectStyle};
use draw::text::TextStyle;
use geometry::Size;
use color::*;

static COLOR_BUTTON_DEFAULT: Color = GRAY_80;
static COLOR_BUTTON_PRESSED: Color = GRAY_60;
static COLOR_BUTTON_ACTIVATED: Color = GRAY_40;
static COLOR_BUTTON_ACTIVATED_PRESSED: Color = GRAY_30;
static COLOR_BUTTON_INACTIVE: Color = GRAY_90;
static COLOR_BUTTON_MOUSEOVER: Color = GRAY_90;
static COLOR_BUTTON_TEXT_INACTIVE: Color = GRAY_70;

static BUTTON_BORDER: (f32, Color) = (1.0, GRAY_40);
static BUTTON_BORDER_INACTIVE: (f32, Color) = (1.0, GRAY_70);


lazy_static! {
    pub static ref STYLE_BUTTON: Vec<RectStyle> = {
        style!(
            RectStyle::BackgroundColor: selector!(COLOR_BUTTON_DEFAULT,
                ACTIVATED_PRESSED: COLOR_BUTTON_ACTIVATED_PRESSED,
                ACTIVATED: COLOR_BUTTON_ACTIVATED,
                PRESSED: COLOR_BUTTON_PRESSED,
                MOUSEOVER: COLOR_BUTTON_MOUSEOVER,
                INACTIVE: COLOR_BUTTON_INACTIVE),
            RectStyle::CornerRadius: Some(5.0),
            RectStyle::Border: selector!(Some(BUTTON_BORDER),
                INACTIVE: Some(BUTTON_BORDER_INACTIVE))
        )
    };
    pub static ref STYLE_BUTTON_TEXT: Vec<TextStyle> = {
        style!(TextStyle::TextColor: selector!(BLACK, INACTIVE: COLOR_BUTTON_TEXT_INACTIVE))
    };
}

// show whether button is held down or not
fn button_handle_mouse_down(event: &WidgetMouseButton, mut args: EventArgs) {
    if !args.widget.props().contains(&Property::Inactive) {
        let &WidgetMouseButton(state, _) = event;
        match state {
            glutin::ElementState::Pressed => args.widget.add_prop(Property::Pressed),
            glutin::ElementState::Released => args.widget.remove_prop(Property::Pressed),
        }
    }
}

pub enum ToggleEvent {
    On,
    Off,
}
// show whether toggle button is activated
fn toggle_button_handle_mouse(event: &WidgetMouseButton, mut args: EventArgs) {
    if let WidgetMouseButton(glutin::ElementState::Released, _) = *event {
        let activated = args.widget.props().contains(&Property::Activated);
        if activated {
            args.widget.event(ToggleEvent::Off);
            args.widget.remove_prop(Property::Activated);
        } else {
            args.widget.event(ToggleEvent::On);
            args.widget.add_prop(Property::Activated);
        }
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
            .set_draw_state_with_style(RectState::new(), STYLE_BUTTON.clone())
            .add_handler_fn(button_handle_mouse_down)
            .enable_hover()
            .add_handler_fn(toggle_button_handle_mouse);
        widget.layout().add(constraints![
            min_size(Size::new(70.0, 30.0)),
            shrink(),
        ]);

        ToggleButtonBuilder { widget: widget }
    }
    pub fn set_text(&mut self, on_text: &'static str, off_text: &'static str) -> &mut Self {

        let style = style!(parent: STYLE_BUTTON_TEXT,
            TextStyle::Text: selector!(off_text.to_owned(),
                ACTIVATED: on_text.to_owned()),
            TextStyle::Align: Align::Middle);
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
        where F: Fn(&ToggleEvent, EventArgs) + 'static
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
            .set_draw_state_with_style(RectState::new(), STYLE_BUTTON.clone())
            .add_handler_fn(button_handle_mouse_down)
            .enable_hover();
        widget.layout().add(constraints![
            min_size(Size::new(100.0, 50.0)).strength(STRONG),
            shrink(),
        ]);

        PushButtonBuilder { widget: widget }
    }
    pub fn set_text(&mut self, text: &'static str) -> &mut Self {

        let style = style!(parent: STYLE_BUTTON_TEXT,
            TextStyle::Text: text.to_owned(),
            TextStyle::Align: Align::Middle);

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
        where F: Fn(&ClickEvent, &mut EventArgs) + 'static
    {
        self.add_handler_fn(move |event, mut args| {
            (on_click)(event, &mut args);
            *args.handled = true;
        })
    }
}
