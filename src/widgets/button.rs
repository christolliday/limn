use glutin;

use text_layout::Align;
use cassowary::strength::*;

use layout::constraint::*;
use event::EventArgs;
use widget::WidgetBuilder;
use widget::property::Property;
use widget::property::states::*;
use widgets::text::TextBuilder;
use input::mouse::WidgetMouseButton;
use draw::rect::{RectState, RectStyle};
use draw::text::TextStyle;
use geometry::Size;
use color::*;
use widget::draw::Draw;
use widget::style::Style;
use resources::font::Font;
use std::sync::Arc;

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

// --------------------- end of default functions

// The WidgetBuilder is getting built only when it is needed!
// The reason for this is configurability. When pulling out all configuration
// options into a `Builder`, the library user can more easily configure
// which things he wants to override. Plus, we are not calling unnecessary
// methods, if the widget ends up unused for some reason.

// Between the time where the ToggleButtonBuilder is constructed
// and the time where the ToggleButtonBuilder is converted into a WidgetBuilder
// the user can configure the ToggleButtonBuilder by overriding the fields directly

/// Padding for a button (or other elements)
#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub struct Padding {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

/// Button builder, shared between `ToggleButtonBuilder`
/// and `PushButtonBuilder`
pub struct ButtonBuilder<F, D, S>
    where F: Fn(&ToggleEvent, EventArgs) + 'static,
          D: Draw + 'static,
          S: Style<D> + 'static {
    /// name, such as "button_text", "toggle_button"
    pub name: &'static str,
    pub hover_enabled: bool,
    pub min_size_horz: f32,
    pub min_size_vert: f32,
    pub on_toggle_handler: Option<F>,
    pub style: Option<(D, S)>,
}

impl<F, D, S> ButtonBuilder<F, D, S>
    where F: Fn(&ToggleEvent, EventArgs) + 'static,
          D: Draw + 'static,
          S: Style<D> + 'static
{
    /// Creates a new button. The button does not know anything about
    /// any containing text. 
    pub fn new(name: &'static str)
               -> Self
    {
        Self {
            name: name,
            hover_enabled: true,
            min_size_horz: 70.0,
            min_size_vert: 30.0,
            on_toggle_handler: None,
            style: Some((RectState::new(), STYLE_BUTTON.clone())),
        }
    }
    
    /// Sets the callback function 
    pub fn on_toggle(self, callback: F) -> Self
    {
        self.on_toggle_handler = Some(callback);
        self
    }
}

impl<F, D, S> Into<WidgetBuilder> for ButtonBuilder<F, D, S>
    where F: Fn(&ToggleEvent, EventArgs) + 'static,
          D: Draw + 'static,
          S: Style<D> + 'static
{
    fn into(self) -> WidgetBuilder {

        let mut widget = WidgetBuilder::new(self.name);

        if self.hover_enabled {
            widget.enable_hover();
        }
        
        if let Some((state, style)) = self.style {
            widget.set_draw_state_with_style(state, style);
        }

        if let Some(handler) = self.on_toggle_handler {
            widget.add_handler_fn(handler);
        }

        widget.layout().add(constraints![
            min_size(Size::new(self.min_size_horz, self.min_size_vert)),
            shrink(),
        ]);

        if let Some(callback) = self.on_toggle_handler {
            widget.add_handler_fn(callback);
        }
        
        widget
    }
}

/// Toggle button, specialization of the generic 
pub struct ToggleButtonBuilder<F, D, S>
    where F: Fn(&ToggleEvent, EventArgs) + 'static,
          D: Draw + 'static,
          S: Style<D> + 'static {
    /// The underlying button builder
    pub button_builder: ButtonBuilder<F, D, S>,
    pub on_off_text: Option<(String, String, Padding, Arc<Font>)>,
}

impl<F, D, S> ToggleButtonBuilder<F, D, S>
    where F: Fn(&ToggleEvent, EventArgs) + 'static,
          D: Draw + 'static,
          S: Style<D> + 'static    
{
    /// Creates a new ToggleButtonBuilder
    pub fn new()
               -> Self
    {
        Self {
            button_builder: ButtonBuilder::new("toggle_button"),
            on_off_text: None,
        }
    }
    
    /// Builder method to tell if there should be a text displayed on the button
    /// The `on_text` is shown when the button is in its "On" state and vice versa.
    #[inline]
    pub fn with_text<T>(self, on_text: T, off_text: T, font: Arc<Font>)
                        -> Self where T: Into<String>
    {
        self.on_off_text = Some((
            on_text.into(),
            off_text.into(),
            Padding {
                left: 20.0,
                right: 20.0,
                top: 10.0,
                bottom: 10.0,
            },
            font));
        
        self
    }
}

impl<F, D, S> Into<WidgetBuilder> for ToggleButtonBuilder<F, D, S>
    where F: Fn(&ToggleEvent, EventArgs) + 'static,
          D: Draw + 'static,
          S: Style<D> + 'static
{
    fn into(self) -> WidgetBuilder {
        
        let mut toggle_button: WidgetBuilder = self.button_builder.into();

        // now apply styles for on and off text if necessary
        if self.on_off_text.is_none() { return toggle_button; }
        let (on_text, off_text, padding, font) = self.on_off_text.unwrap();

        let style = style!(
            parent: STYLE_BUTTON_TEXT,
            TextStyle::Text: selector!(off_text,
                                       ACTIVATED: on_text),
            TextStyle::Align: Align::Middle
        );
        
        let mut button_text_widget: WidgetBuilder =
            TextBuilder::new(off_text, font)
            .with_style(style)
            .into();

        button_text_widget.set_name("button_text");
        
        button_text_widget.layout().add(constraints![
            bound_left(&toggle_button).padding(padding.left),
            bound_right(&toggle_button).padding(padding.right),
            bound_top(&toggle_button).padding(padding.top),
            bound_bottom(&toggle_button).padding(padding.bottom),
            center(&toggle_button),
        ]);

        toggle_button.add_child(button_text_widget);

        toggle_button
    }
}

/// PushButtonBuilder, a specialization of the ButtonBuilder
/// that only displays a text and doesn't change it in any way
pub struct PushButtonBuilder<F, D, S>
    where F: Fn(&ToggleEvent, EventArgs) + 'static,
          D: Draw + 'static,
          S: Style<D> + 'static
{
    /// The underlying button builder, which does not know anything
    /// about the font
    pub button_builder: ButtonBuilder<F, D, S>,
    /// The text to be displayed on the button
    pub text: Option<(String, Padding, Arc<Font>)>,
}

impl<F, D, S> PushButtonBuilder<F, D, S>
    where F: Fn(&ToggleEvent, EventArgs) + 'static,
          D: Draw + 'static,
          S: Style<D> + 'static    
{
    
    pub fn new() -> Self {
        Self {
            button_builder: ButtonBuilder::new("push_button"),
            text: None,
        }
    }

    /// Sets the text to display on a PushButtonBuilder
    #[inline]
    pub fn with_text<T>(self, text: T, off_text: T, font: Arc<Font>)
                        -> Self where T: Into<String>
    {
        self.text = Some((
            text.into(),
            Padding {
                left: 20.0,
                right: 20.0,
                top: 10.0,
                bottom: 10.0,
            },
            font));
        
        self
    }
}

impl<F, D, S> Into<WidgetBuilder> for PushButtonBuilder<F, D, S>
    where F: Fn(&ToggleEvent, EventArgs) + 'static,
          D: Draw + 'static,
          S: Style<D> + 'static
{
    fn into(self) -> WidgetBuilder {
        
        let push_button: WidgetBuilder = self.button_builder.into();

        // add text as a child only if needed
        if self.text.is_none() { return push_button; }
        let (text, padding, font) = self.text.unwrap();

        let style = style!(
            parent: STYLE_BUTTON_TEXT,
            TextStyle::Text: text,
            TextStyle::Align: Align::Middle
        );
        
        let mut button_text_widget: WidgetBuilder =
            TextBuilder::new(text, font)
            .with_style(style)
            .into();

        button_text_widget.set_name("button_text");
        button_text_widget.layout().add(constraints![
            bound_left(&push_button).padding(20.0),
            bound_right(&push_button).padding(20.0),
            bound_top(&push_button).padding(10.0),
            bound_bottom(&push_button).padding(10.0),
            center(&push_button),
        ]);

        push_button.add_child(button_text_widget);
        push_button
    }
}
