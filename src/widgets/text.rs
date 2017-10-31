use cassowary::Constraint;

use widget::WidgetBuilder;
use widget::style::StyleUpdated;
use draw::text::{TextState, TextStyle};
use event::{EventHandler, EventArgs};
use layout::constraint::*;

#[derive(Default)]
struct TextUpdatedHandler {
    size_constraints: Vec<Constraint>,
}
impl EventHandler<StyleUpdated> for TextUpdatedHandler {
    fn handle(&mut self, _: &StyleUpdated, mut args: EventArgs) {
        args.widget.update_layout(|layout| {
            for constraint in self.size_constraints.drain(..) {
                layout.remove_constraint(constraint);
            }
        });
        let text_size = {
            let draw_state = args.widget.draw_state();
            let text_draw_state = draw_state.downcast_ref::<TextState>().unwrap();
            text_draw_state.measure()
        };
        let size_constraints = size(text_size).build(&args.widget.layout_vars());
        args.widget.update_layout(|layout| {
            layout.add(size_constraints.clone());
        });
        self.size_constraints = size_constraints;
    }
}

use style::*;

#[derive(Clone)]
pub struct TextComponent {
    pub style: Option<Vec<TextStyle>>,
}
impl TextComponent {
    pub fn text(&mut self, text: &str) {
        self.style = Some(style!(TextStyle::Text: text.to_owned()));
    }
    pub fn style(&mut self, style: Vec<TextStyle>) {
        self.style = Some(style);
    }
}

impl Default for TextComponent {
    fn default() -> Self {
        TextComponent {
            style: Some(vec![]),
        }
    }
}

impl Component for TextComponent {
    type Values = TextComponentValues;
    fn name() -> String {
        "text".to_owned()
    }
    fn merge(&self, other: &Self) -> Self {
        TextComponent {
            style: self.style.merge(&other.style),
        }
    }
    fn to_values(self) -> Self::Values {
        TextComponentValues {
            style: self.style.unwrap(),
        }
    }
}

#[derive(Debug)]
pub struct TextComponentValues {
    style: Vec<TextStyle>,
}

impl ComponentValues for TextComponentValues {
    fn apply(&self, widget: &mut WidgetBuilder) {
        let text_draw_state = TextState::default();
        widget.set_draw_state_with_style(text_draw_state, self.style.clone());
        widget.add_handler(TextUpdatedHandler::default());
    }
}
