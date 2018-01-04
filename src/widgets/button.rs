use std::any::TypeId;

use glutin;

use layout::constraint::*;
use event::EventArgs;
use widget::Widget;
use widget::property::Property;
use input::mouse::WidgetMouseButton;
use widgets::text::StaticTextStyle;
use draw::rect::RectStyle;
use draw::text::TextStyle;
use geometry::Size;
use style::*;
use widget::property::states::*;

component_style!{pub struct Button<name="button", style=ButtonStyle> {
    rect: RectStyle = RectStyle::default(),
    text: Option<TextStyle> = None,
}}

impl ButtonStyle {
    pub fn from_text(text: &str) -> Self {
        Self {
            text: Some(Some(TextStyle::from_text(text))),
            ..Self::default()
        }
    }
}

impl WidgetModifier for Button {
    fn apply(&self, widget: &mut Widget) {
        widget
            .set_style_class(TypeId::of::<RectStyle>(), "button_rect")
            .set_draw_style(self.rect.clone())
            .enable_press()
            .enable_hover();
        widget.layout().add(constraints![
            min_size(Size::new(70.0, 30.0)),
            shrink(),
        ]);
        if let Some(text_style) = self.text.clone() {
            let mut button_text_widget = Widget::new("button_text");
            button_text_widget.set_style_class(TypeId::of::<TextStyle>(), "button_text");
            StaticTextStyle::from_style(text_style).component().apply(&mut button_text_widget);
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

component_style!{pub struct ToggleButton<name="toggle_button", style=ToggleButtonStyle> {
    rect: RectStyle = RectStyle::default(),
    off_text: Option<TextStyle> = None,
    on_text: Option<TextStyle> = None,
}}

impl ToggleButtonStyle {
    pub fn rect_style(&mut self, rect: RectStyle) {
        self.rect = Some(rect);
    }
    pub fn text(&mut self, text: &str) {
        self.off_text = Some(Some(style!(TextStyle {
            text: String::from(text)
        })));
    }
    pub fn toggle_text(&mut self, off_text: &str, on_text: &str) {
        self.off_text = Some(Some(style!(TextStyle {
            text: String::from(off_text)
        })));
        self.on_text = Some(Some(style!(TextStyle {
            text: String::from(on_text)
        })));
    }
}

impl WidgetModifier for ToggleButton {
    fn apply(&self, widget: &mut Widget) {
        widget
            .set_style_class(TypeId::of::<RectStyle>(), "button_rect")
            .set_draw_style(self.rect.clone())
            .enable_press()
            .enable_hover();
        widget.layout().add(constraints![
            min_size(Size::new(70.0, 30.0)),
            shrink(),
        ]);
        let mut button_text_widget = Widget::new("button_text");
        button_text_widget.set_style_class(TypeId::of::<TextStyle>(), "button_text");
        button_text_widget.layout().add(constraints![
            bound_left(widget).padding(20.0),
            bound_right(widget).padding(20.0),
            bound_top(widget).padding(10.0),
            bound_bottom(widget).padding(10.0),
            center(widget),
        ]);
        if let Some(ref text_style) = self.off_text {
            StaticTextStyle::from_style(text_style.clone()).component().apply(&mut button_text_widget);
        }
        if let Some(text_style) = self.on_text.clone() {
            button_text_widget.set_draw_style_prop(ACTIVATED.clone(), text_style);
        }
        widget.add_child(button_text_widget);
        widget.enable_toggle();
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ToggleEvent {
    On,
    Off,
}

impl Widget {
    fn enable_press(&mut self) -> &mut Self {
        self.add_handler(|event: &WidgetMouseButton, mut args: EventArgs| {
            if !args.widget.props().contains(&Property::Inactive) {
                let &WidgetMouseButton(state, _) = event;
                match state {
                    glutin::ElementState::Pressed => args.widget.add_prop(Property::Pressed),
                    glutin::ElementState::Released => args.widget.remove_prop(Property::Pressed),
                }
            }
        })
    }
    fn enable_toggle(&mut self) -> &mut Self {
        self.add_handler(|event: &WidgetMouseButton, mut args: EventArgs| {
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
        })
    }
}
