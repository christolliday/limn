use cassowary::Constraint;

use widget::WidgetBuilder;
use widget::style::StyleUpdated;
use draw::text::{TextState, TextStyle};
use event::{EventHandler, EventArgs};
use layout::constraint::*;
use resources::font::Font;
use std::sync::Arc;

pub struct TextBuilder {
    pub text: String,
    pub font: Arc<Font>,
    pub style: Option<Vec<TextStyle>>,
}

impl TextBuilder {

    /// Create a new text builder. The actual widget will not be
    /// created until the TextBuilder is actually consumed
    #[inline]
    pub fn new<S>(text: S, font: Arc<Font>)
                  -> Self where S: Into<String>
    {
        Self {
            text: text.into(),
            font: font,
            style: None,
        }
    }

    /// Builder method to change the default style
    #[inline]
    pub fn with_style(self, style: Vec<TextStyle>)
                      -> Self
    {
        self.style = Some(style);
        self
    }
}

impl Into<WidgetBuilder> for TextBuilder {
    fn into(self) -> WidgetBuilder {
        let text_draw_state = TextState::new(self.text.clone(), self.font);
        let mut widget = WidgetBuilder::new(self.text);

        if let Some(text_style) = self.style {
            widget.set_draw_state_with_style(text_draw_state, text_style);            
        } else {
            widget.set_draw_state(text_draw_state);
        }

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
