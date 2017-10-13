use cassowary::Constraint;

use layout::constraint::ConstraintBuilder;
use layout::constraint::*;
use widget::style::StyleUpdated;
use widget::WidgetBuilder;
use widget::property::states::*;
use ui::{WidgetAttachedEvent, WidgetDetachedEvent};
use input::keyboard::{WidgetReceivedCharacter, KeyboardInputEvent};
use draw::rect::{RectState, RectStyle};
use draw::text::TextState;
use event::{EventHandler, EventArgs};
use color::*;

const BACKSPACE: char = '\u{8}';

fn edit_text_handle_char(event: &WidgetReceivedCharacter, mut args: EventArgs) {
    let &WidgetReceivedCharacter(char) = event;
    let text = {
        let bounds = args.widget.bounds();
        let draw_state = args.widget.draw_state();
        let text_draw_state = draw_state.downcast_ref::<TextState>().unwrap();
        let mut text = text_draw_state.text.clone();
        match char {
            BACKSPACE => {
                text.pop();
            }
            _ => {
                text.push(char);
                if !text_draw_state.text_fits(&text, bounds) {
                    text.pop();
                }
            }
        }
        text
    };
    args.widget.update(|state: &mut TextState| {
        state.text = text.clone()
    });
    args.widget.event(TextUpdated(text.clone()));
}

pub struct TextUpdated(pub String);

pub fn text_change_handle(event: &TextUpdated, mut args: EventArgs) {
    args.widget.update(|state: &mut TextState| state.text = event.0.clone());
}

pub struct EditTextBuilder {
    pub widget: WidgetBuilder,
    pub text_widget: WidgetBuilder,
}

impl Default for EditTextBuilder {
    fn default() -> Self {
        let default_border = Some((1.0, GRAY_70));
        let focused_border = Some((1.0, BLUE));
        let rect_style = style!(
            RectStyle::Border: selector!(default_border, FOCUSED: focused_border),
            RectStyle::CornerRadius: Some(3.0));
        let mut widget = WidgetBuilder::new("edit_text");
        widget
            .set_draw_state_with_style(RectState::new(), rect_style)
            .add_handler_fn(|_: &WidgetAttachedEvent, args| {
                args.ui.event(KeyboardInputEvent::AddFocusable(args.widget));
            })
            .add_handler_fn(|_: &WidgetDetachedEvent, args| {
                args.ui.event(KeyboardInputEvent::RemoveFocusable(args.widget));
            })
            .make_focusable();

        let mut text_widget = WidgetBuilder::new("edit_text_text");
        text_widget
            .set_draw_state(TextState::default())
            .add_handler(TextUpdatedHandler::default())
            .add_handler_fn(edit_text_handle_char)
            .add_handler_fn(text_change_handle);

        text_widget.layout().add(constraints![
            align_left(&widget).padding(5.0),
            align_top(&widget).padding(5.0),
            bound_by(&widget).padding(5.0),
        ]);

        EditTextBuilder {
            widget: widget,
            text_widget: text_widget,
        }
    }
}
impl EditTextBuilder {
    /// Creates a new `EditTextBuilder`
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the callback which is used on changing the edited text
    pub fn on_text_changed<F>(&mut self, callback: F) -> &mut Self
        where F: Fn(&TextUpdated, EventArgs) + 'static
    {
        self.text_widget.add_handler_fn(callback);
        self
    }
}

widget_builder!(EditTextBuilder);

impl Into<WidgetBuilder> for EditTextBuilder {
    fn into(mut self) -> WidgetBuilder {
        self.widget.add_child(self.text_widget);
        self.widget
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
        let line_height = {
            let draw_state = args.widget.draw_state();
            let text_draw_state = draw_state.downcast_ref::<TextState>().unwrap();
            text_draw_state.line_height()
        };
        let size_constraints = min_height(line_height).build(&args.widget.layout_vars());
        args.widget.update_layout(|layout| {
            layout.add(size_constraints.clone())
        });
        self.size_constraints = size_constraints;
    }
}
