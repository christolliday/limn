use glutin;

use layout::constraint::*;
use event::EventArgs;
use widget::WidgetBuilder;
use widget::property::Property;
use widget::property::states::*;
use widget::style::Value;
use input::mouse::WidgetMouseButton;
use widgets::text::StaticTextStyle;
use draw::rect::RectComponentStyle;
use draw::text::TextComponentStyle;
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
    pub static ref STYLE_BUTTON_RECT: RectComponentStyle = RectComponentStyle {
        background_color: Some(Value::from(selector!(COLOR_BUTTON_DEFAULT,
            ACTIVATED_PRESSED: COLOR_BUTTON_ACTIVATED_PRESSED,
            ACTIVATED: COLOR_BUTTON_ACTIVATED,
            PRESSED: COLOR_BUTTON_PRESSED,
            MOUSEOVER: COLOR_BUTTON_MOUSEOVER,
            INACTIVE: COLOR_BUTTON_INACTIVE))),
        corner_radius: Some(Value::from(Some(5.0))),
        border: Some(Value::from(selector!(Some(BUTTON_BORDER),
            INACTIVE: Some(BUTTON_BORDER_INACTIVE)))),
    };

    pub static ref STYLE_BUTTON_TEXT: TextComponentStyle = TextComponentStyle {
        text_color: Some(Value::from(selector!(BLACK, INACTIVE: COLOR_BUTTON_TEXT_INACTIVE))),
        ..TextComponentStyle::default()
    };
}

#[derive(Debug, Copy, Clone)]
pub enum ToggleEvent {
    On,
    Off,
}

#[derive(Clone)]
pub struct ButtonStyle {
    rect: Option<RectComponentStyle>,
    text: Option<Option<TextComponentStyle>>,
    toggle: Option<bool>,
}

impl ButtonStyle {
    pub fn rect_style(&mut self, rect: RectComponentStyle) {
        self.rect = Some(rect);
    }
    pub fn text_style(&mut self, text: Option<TextComponentStyle>) {
        self.text = Some(text);
    }
    pub fn text(&mut self, text: &str) {
        self.text = Some(Some(TextComponentStyle {
            text: Some(Value::from(text.to_owned())),
            ..TextComponentStyle::default()
        }));
    }
    pub fn toggle_text(&mut self, on_text: &str, off_text: &str) {
        self.text = Some(Some(TextComponentStyle {
            text: Some(Value::from(selector!(on_text.to_owned(), ACTIVATED: off_text.to_owned()))),
            ..TextComponentStyle::default()
        }));
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
    fn merge(&self, other: &Self) -> Self {
        ButtonStyle {
            rect: self.rect.as_ref().or(other.rect.as_ref()).cloned(),
            text: self.text.as_ref().or(other.text.as_ref()).cloned(),
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

#[derive(Clone)]
pub struct ButtonComponent {
    rect: RectComponentStyle,
    text: Option<TextComponentStyle>,
    toggle: bool,
}

impl Component for ButtonComponent {
    fn name() -> String {
        "button".to_owned()
    }
}

impl WidgetModifier for ButtonComponent {
    fn apply(&self, widget: &mut WidgetBuilder) {
        widget
            .set_style_class("button_rect")
            .set_draw_style(self.rect.clone())
            .add_handler(|event: &WidgetMouseButton, mut args: EventArgs| {
                if !args.widget.props().contains(&Property::Inactive) {
                    let &WidgetMouseButton(state, _) = event;
                    match state {
                        glutin::ElementState::Pressed => args.widget.add_prop(Property::Pressed),
                        glutin::ElementState::Released => args.widget.remove_prop(Property::Pressed),
                    }
                }
            })
            .enable_hover();
        widget.layout().add(constraints![
            min_size(Size::new(70.0, 30.0)),
            shrink(),
        ]);
        if let Some(text_style) = self.text.clone() {
            let mut button_text_widget = WidgetBuilder::new("button_text");
            button_text_widget.set_style_class("button_text");
            let text = StaticTextStyle {
                style: Some(text_style),
            };
            text.component().apply(&mut button_text_widget);
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
            widget.add_handler(|event: &WidgetMouseButton, mut args: EventArgs| {
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
            });
        }
    }
}
