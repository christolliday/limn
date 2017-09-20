use cassowary::Constraint;

use widget::WidgetBuilder;
use widget::style::StyleUpdated;
use draw::text::{TextState, TextStyle};
use event::{EventHandler, EventArgs};
use layout::constraint::*;

pub struct TextBuilder;

impl TextBuilder {
    pub fn new(text: &str) -> WidgetBuilder {
        let text_draw_state = TextState::new(text);
        let mut widget = WidgetBuilder::new(text);
        widget.set_draw_state(text_draw_state);
        widget.add_handler(TextUpdatedHandler { size_constraints: Vec::new() });
        widget
    }
    pub fn new_with_style(style: Vec<TextStyle>) -> WidgetBuilder {
        let text_draw_state = TextState::default();
        let mut widget = WidgetBuilder::new("text");
        widget.set_draw_state_with_style(text_draw_state, style);
        widget.add_handler(TextUpdatedHandler::default());
        widget
    }
}

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
