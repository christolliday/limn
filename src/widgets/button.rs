use glutin;

use layout::constraint::*;
use event::EventArgs;
use widget::WidgetBuilder;
use widget::property::Property;
use widget::property::states::*;
use input::mouse::WidgetMouseButton;
use widgets::text::StaticTextStyle;
use draw::rect::{RectState, RectStyle};
use draw::text::TextStyle;
use geometry::Size;
use color::*;
use style::*;

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
    pub static ref STYLE_BUTTON_RECT: Vec<RectStyle> = {
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

/// Show whether button is held down or not
pub fn button_handle_mouse_down(event: &WidgetMouseButton, mut args: EventArgs) {
    if !args.widget.props().contains(&Property::Inactive) {
        let &WidgetMouseButton(state, _) = event;
        match state {
            glutin::ElementState::Pressed => args.widget.add_prop(Property::Pressed),
            glutin::ElementState::Released => args.widget.remove_prop(Property::Pressed),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ToggleEvent {
    On,
    Off,
}

/// Show whether toggle button is activated
pub fn toggle_button_handle_mouse(event: &WidgetMouseButton, mut args: EventArgs) {
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

#[derive(Clone)]
pub struct ButtonStyle {
    rect: Option<Vec<RectStyle>>,
    text: Option<Option<Vec<TextStyle>>>,
    toggle: Option<bool>,
}

impl ButtonStyle {
    pub fn rect_style(&mut self, rect: Vec<RectStyle>) {
        self.rect = Some(rect);
    }
    pub fn text_style(&mut self, text: Option<Vec<TextStyle>>) {
        self.text = Some(text);
    }
    pub fn text(&mut self, text: &str) {
        self.text = Some(Some(style!(TextStyle::Text: text.to_owned())));
    }
    pub fn toggle_text(&mut self, on_text: &str, off_text: &str) {
        self.text = Some(Some(style!(TextStyle::Text:
            selector!(on_text.to_owned(), ACTIVATED: off_text.to_owned()))));
        self.toggle = Some(true);
    }
    pub fn toggle(&mut self, toggle: bool) {
        self.toggle = Some(toggle);
    }
}

impl Default for ButtonStyle {
    fn default() -> Self {
        ButtonStyle {
            rect: Some(STYLE_BUTTON_RECT.clone()),
            text: Some(None),
            toggle: Some(false),
        }
    }
}

impl ComponentStyle for ButtonStyle {
    type Component = ButtonComponent;
    fn name() -> String {
        "button".to_owned()
    }
    fn merge(&self, other: &Self) -> Self {
        ButtonStyle {
            rect: self.rect.as_ref().or(other.rect.as_ref()).cloned(),
            text: self.text.merge(&other.text),
            toggle: self.toggle.as_ref().or(other.toggle.as_ref()).cloned(),
        }
    }
    fn component(self) -> Self::Component {
        ButtonComponent {
            rect: self.rect.unwrap(),
            text: self.text.unwrap(),
            toggle: self.toggle.unwrap(),
        }
    }
}

#[derive(Debug)]
pub struct ButtonComponent {
    rect: Vec<RectStyle>,
    text: Option<Vec<TextStyle>>,
    toggle: bool,
}

impl Component for ButtonComponent {
    fn apply(&self, widget: &mut WidgetBuilder) {
        widget
            .set_style_class("button_rect")
            .set_draw_state_with_style(RectState::new(), self.rect.clone())
            .add_handler(button_handle_mouse_down)
            .enable_hover();
        widget.layout().add(constraints![
            min_size(Size::new(70.0, 30.0)),
            shrink(),
        ]);
        if let Some(text_style) = self.text.clone() {
            let mut button_text_widget = WidgetBuilder::new("button_text");
            button_text_widget
                .set_style_class("button_text");
            let text = StaticTextStyle {
                style: Some(text_style),
            };
            text.apply(&mut button_text_widget);
            button_text_widget.layout().add(constraints![
                bound_left(widget).padding(20.0),
                bound_right(widget).padding(20.0),
                bound_top(widget).padding(10.0),
                bound_bottom(widget).padding(10.0),
                center(widget),
            ]);

            widget.add_child(button_text_widget);
        }
        if self.toggle {
            widget.add_handler(toggle_button_handle_mouse);
        }
    }
}
