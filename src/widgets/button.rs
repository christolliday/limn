use std::any::TypeId;

use glutin;

use layout::constraint::*;
use event::EventArgs;
use widget::WidgetBuilder;
use widget::property::Property;
use input::mouse::WidgetMouseButton;
use widgets::text::StaticTextStyle;
use draw::rect::RectComponentStyle;
use draw::text::TextComponentStyle;
use geometry::Size;
use style::*;
use widget::property::states::*;

#[derive(Debug, Copy, Clone)]
pub enum ToggleEvent {
    On,
    Off,
}

#[derive(Debug, Clone)]
pub struct ButtonStyle {
    rect: Option<RectComponentStyle>,
    text: Option<Option<TextComponentStyle>>,
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
            text: Some(text.to_owned()),
            ..TextComponentStyle::default()
        }));
    }
}

impl Default for ButtonStyle {
    fn default() -> Self {
        ButtonStyle {
            rect: None,
            text: Some(None),
        }
    }
}

impl ComponentStyle for ButtonStyle {
    type Component = ButtonComponent;
    fn merge(&self, other: &Self) -> Self {
        ButtonStyle {
            rect: self.rect.as_ref().or(other.rect.as_ref()).cloned(),
            text: self.text.as_ref().or(other.text.as_ref()).cloned(),
        }
    }
    fn component(self) -> Self::Component {
        ButtonComponent {
            rect: self.rect.unwrap_or(RectComponentStyle::default()),
            text: self.text.unwrap_or(None),
        }
    }
}

#[derive(Clone)]
pub struct ButtonComponent {
    rect: RectComponentStyle,
    text: Option<TextComponentStyle>,
}

impl Component for ButtonComponent {
    fn name() -> String {
        "button".to_owned()
    }
}

impl WidgetModifier for ButtonComponent {
    fn apply(&self, widget: &mut WidgetBuilder) {
        widget
            .set_style_class(TypeId::of::<RectComponentStyle>(), "button_rect")
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
            button_text_widget.set_style_class(TypeId::of::<TextComponentStyle>(), "button_text");
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
    }
}

#[derive(Debug, Clone)]
pub struct ToggleButtonStyle {
    rect: Option<RectComponentStyle>,
    off_text: Option<Option<TextComponentStyle>>,
    on_text: Option<Option<TextComponentStyle>>,
}

impl ToggleButtonStyle {
    pub fn rect_style(&mut self, rect: RectComponentStyle) {
        self.rect = Some(rect);
    }
    pub fn text(&mut self, text: &str) {
        self.off_text = Some(Some(TextComponentStyle {
            text: Some(text.to_owned()),
            ..TextComponentStyle::default()
        }));
    }
    pub fn toggle_text(&mut self, off_text: &str, on_text: &str) {
        self.off_text = Some(Some(TextComponentStyle {
            text: Some(off_text.to_owned()),
            ..TextComponentStyle::default()
        }));
        self.on_text = Some(Some(TextComponentStyle {
            text: Some(on_text.to_owned()),
            ..TextComponentStyle::default()
        }));
    }
}

impl Default for ToggleButtonStyle {
    fn default() -> Self {
        ToggleButtonStyle {
            rect: None,
            off_text: Some(None),
            on_text: Some(None),
        }
    }
}

impl ComponentStyle for ToggleButtonStyle {
    type Component = ToggleButtonComponent;
    fn merge(&self, other: &Self) -> Self {
        ToggleButtonStyle {
            rect: self.rect.as_ref().or(other.rect.as_ref()).cloned(),
            off_text: self.off_text.as_ref().or(other.off_text.as_ref()).cloned(),
            on_text: self.on_text.as_ref().or(other.on_text.as_ref()).cloned(),
        }
    }
    fn component(self) -> Self::Component {
        ToggleButtonComponent {
            rect: self.rect.unwrap_or(RectComponentStyle::default()),
            off_text: self.off_text.unwrap_or(None),
            on_text: self.on_text.unwrap_or(None),
        }
    }
}

#[derive(Clone)]
pub struct ToggleButtonComponent {
    rect: RectComponentStyle,
    off_text: Option<TextComponentStyle>,
    on_text: Option<TextComponentStyle>,
}

impl Component for ToggleButtonComponent {
    fn name() -> String {
        "button".to_owned()
    }
}
impl WidgetModifier for ToggleButtonComponent {
    fn apply(&self, widget: &mut WidgetBuilder) {
        widget
            .set_style_class(TypeId::of::<RectComponentStyle>(), "button_rect")
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
        let mut button_text_widget = WidgetBuilder::new("button_text");
        button_text_widget.set_style_class(TypeId::of::<TextComponentStyle>(), "button_text");
        button_text_widget.layout().add(constraints![
            bound_left(widget).padding(20.0),
            bound_right(widget).padding(20.0),
            bound_top(widget).padding(10.0),
            bound_bottom(widget).padding(10.0),
            center(widget),
        ]);
        if let Some(text_style) = self.off_text.clone() {
            let mut style = StaticTextStyle::default();
            style.style(text_style);
            style.component().apply(&mut button_text_widget);
        }
        if let Some(text_style) = self.on_text.clone() {
            button_text_widget.set_draw_style_prop(ACTIVATED.clone(), text_style);
        }
        widget.add_child(button_text_widget);
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
