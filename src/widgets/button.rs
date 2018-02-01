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
        let mut draw_style = DrawStyle::from(self.rect.clone());
        draw_style.set_class("button_rect");
        widget
            .set_draw_style(draw_style)
            .enable_press()
            .enable_hover();
        widget.layout().add(constraints![
            min_size(Size::new(70.0, 30.0)),
            shrink(),
        ]);
        if let Some(text_style) = self.text.clone() {
            let mut button_text_widget = Widget::new("button_text");
            button_text_widget.set_draw_style(DrawStyle::from_class::<TextStyle>("button_text"));
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
    off_text: TextStyle = TextStyle::default(),
    on_text: Option<TextStyle> = None,
}}

impl ToggleButtonStyle {
    pub fn rect_style(&mut self, rect: RectStyle) {
        self.rect = Some(rect);
    }
    pub fn text(&mut self, text: &str) {
        self.off_text = Some(style!(TextStyle {
            text: String::from(text),
        }));
    }
    pub fn toggle_text(&mut self, on_text: &str, off_text: &str) {
        self.off_text = Some(style!(TextStyle {
            text: String::from(off_text),
        }));
        self.on_text = Some(Some(style!(TextStyle {
            text: String::from(on_text),
        })));
    }
}

impl WidgetModifier for ToggleButton {
    fn apply(&self, widget: &mut Widget) {
        let mut draw_style = DrawStyle::from(self.rect.clone());
        draw_style.set_class("button_rect");
        widget
            .set_draw_style(draw_style)
            .enable_press()
            .enable_hover();
        widget.layout().add(constraints![
            min_size(Size::new(70.0, 30.0)),
            shrink(),
        ]);
        let mut button_text_widget = Widget::new("button_text");
        button_text_widget.set_draw_style(DrawStyle::from_class::<TextStyle>("button_text"));
        button_text_widget.layout().add(constraints![
            bound_left(widget).padding(20.0),
            bound_right(widget).padding(20.0),
            bound_top(widget).padding(10.0),
            bound_bottom(widget).padding(10.0),
            center(widget),
        ]);
        StaticTextStyle::from_style(self.off_text.clone()).component().apply(&mut button_text_widget);

        if let Some(on_style) = self.on_text.clone() {
            button_text_widget.draw_state().style().unwrap().prop_style(ACTIVATED.clone(), on_style);
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
