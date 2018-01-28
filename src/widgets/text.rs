use cassowary::Constraint;

use widget::{Widget, StateUpdated, StyleUpdated};
use draw::text::{TextState, TextStyle};
use event::{EventHandler, EventArgs};
use layout::constraint::*;
use geometry::Size;
use style::WidgetModifier;
use widgets::edit_text::TextUpdated;

component_style!{pub struct StaticText<name="static_text", style=StaticTextStyle> {
    style: TextStyle = TextStyle::default(),
}}

impl StaticTextStyle {
    pub fn from_style(style: TextStyle) -> Self {
        StaticTextStyle {
            style: Some(style),
        }
    }
    pub fn from_text(text: &str) -> Self {
        StaticTextStyle {
            style: Some(style!(TextStyle {
                text: String::from(text),
            })),
        }
    }
}

impl WidgetModifier for StaticText {
    fn apply(&self, widget: &mut Widget) {
        widget.add_handler(TextSizeHandler::default());
        widget.add_handler(TextUpdateHandler::default());
        widget.add_handler(|_: &StyleUpdated, args: EventArgs| {
            args.widget.event(StaticTextUpdate::StyleUpdated);
        });
        widget.add_handler(|event: &TextUpdated, args: EventArgs| {
            args.widget.event(StaticTextUpdate::TextUpdated(event.clone()));
        });
        widget.set_draw_style(self.style.clone());
    }
}

#[derive(Default)]
struct TextUpdateHandler {
    text: Option<String>,
}

enum StaticTextUpdate {
    StyleUpdated,
    TextUpdated(TextUpdated),
}

impl EventHandler<StaticTextUpdate> for TextUpdateHandler {
    fn handle(&mut self, event: &StaticTextUpdate, mut args: EventArgs) {
        match *event {
            StaticTextUpdate::TextUpdated(ref event) => {
                self.text = Some(event.0.clone());
                args.widget.update(|state: &mut TextState| {
                    state.text = event.0.clone();
                });
            },
            StaticTextUpdate::StyleUpdated => {
                if let Some(ref text) = self.text {
                    args.widget.update(|state: &mut TextState| {
                        state.text = text.clone();
                    });
                }
            }
        }
    }
}

#[derive(Default)]
struct TextSizeHandler {
    measured_size: Option<Size>,
    size_constraints: Vec<Constraint>,
}

impl EventHandler<StateUpdated> for TextSizeHandler {
    fn handle(&mut self, _: &StateUpdated, mut args: EventArgs) {
        let text_size = {
            let draw_state = args.widget.draw_state();
            if let Some(state) = draw_state.downcast_ref::<TextState>() {
                state.measure()
            } else {
                Size::zero()
            }
        };
        if self.measured_size.is_none() || self.measured_size.unwrap() != text_size {
            let size_constraints = size(text_size).build(&args.widget.layout_vars());
            let mut layout = args.widget.layout();
            for constraint in self.size_constraints.drain(..) {
                layout.remove_constraint(constraint);
            }
            layout.add(size_constraints.clone());
            self.size_constraints = size_constraints;
            self.measured_size = Some(text_size);
        }
    }
}
