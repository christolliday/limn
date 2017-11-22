use cassowary::Constraint;

use widget::{WidgetBuilder, StateUpdated};
use draw::text::{TextState, TextStyle};
use event::{EventHandler, EventArgs};
use layout::constraint::*;
use geometry::Size;
use style::WidgetModifier;

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
            style: Some(TextStyle {
                text: Some(text.to_owned()),
                ..TextStyle::default()
            }),
        }
    }
}

impl WidgetModifier for StaticText {
    fn apply(&self, widget: &mut WidgetBuilder) {
        widget.add_handler(TextSizeHandler::default());
        widget.set_draw_style(self.style.clone());
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
            let text_draw_state = draw_state.downcast_ref::<TextState>().unwrap();
            text_draw_state.measure()
        };
        if self.measured_size.is_none() || self.measured_size.unwrap() != text_size {
            let size_constraints = size(text_size).build(&args.widget.layout_vars());
            args.widget.update_layout(|layout| {
                for constraint in self.size_constraints.drain(..) {
                    layout.remove_constraint(constraint);
                }
                layout.add(size_constraints.clone());
            });
            self.size_constraints = size_constraints;
            self.measured_size = Some(text_size);
        }
    }
}
