use cassowary::Constraint;

use widget::WidgetBuilder;
use widget::style::StyleUpdated;
use draw::text::{TextState, TextComponentStyle};
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
pub struct StaticTextStyle {
    pub style: Option<TextComponentStyle>,
}
impl StaticTextStyle {
    pub fn text(&mut self, text: &str) {
        self.style = Some(TextComponentStyle {
            text: Some(text.to_owned()),
            ..TextComponentStyle::default()
        });
    }
    pub fn style(&mut self, style: TextComponentStyle) {
        self.style = Some(style);
    }
}

impl Default for StaticTextStyle {
    fn default() -> Self {
        StaticTextStyle {
            style: None,
        }
    }
}

impl ComponentStyle for StaticTextStyle {
    type Component = StaticTextComponent;
    fn merge(&self, other: &Self) -> Self {
        StaticTextStyle {
            style: self.style.as_ref().or(other.style.as_ref()).cloned(),
        }
    }
    fn component(self) -> Self::Component {
        StaticTextComponent {
            style: self.style.unwrap(),
        }
    }
}

#[derive(Clone)]
pub struct StaticTextComponent {
    style: TextComponentStyle,
}

impl Component for StaticTextComponent {
    fn name() -> String {
        "text".to_owned()
    }
}

impl WidgetModifier for StaticTextComponent {
    fn apply(&self, widget: &mut WidgetBuilder) {
        widget.set_draw_style(self.style.clone());
        widget.add_handler(TextUpdatedHandler::default());
    }
}
