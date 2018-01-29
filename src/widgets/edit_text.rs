use cassowary::Constraint;

use layout::constraint::ConstraintBuilder;
use layout::constraint::*;
use widget::{Widget, StyleUpdated};
use ui::{WidgetAttachedEvent, WidgetDetachedEvent};
use input::keyboard::{WidgetReceivedCharacter, KeyboardInputEvent};
use draw::rect::RectStyle;
use draw::text::{TextState, TextStyle};
use event::{EventHandler, EventArgs};
use color::*;
use widget::property::states::*;
use widget::style::DrawStyle;
use style::WidgetModifier;

const BACKSPACE: char = '\u{8}';

#[derive(Debug, Clone)]
pub struct TextUpdated(pub String);

multi_event!{impl EventHandler<EditTextEvent> for EditTextHandler {
    WidgetReceivedCharacter => received_char,
    TextUpdated => text_updated,
    StyleUpdated => style_updated,
}}

struct EditTextHandler {
    text_box: Widget,
    text: String,
}

impl EditTextHandler {
    fn update_text(&mut self) {
        let text = self.text.clone();
        self.text_box.update(|state: &mut TextState| {
            state.text = text;
        });
    }

    fn received_char(&mut self, event: &WidgetReceivedCharacter, args: EventArgs) {
        let &WidgetReceivedCharacter(char) = event;
        match char {
            BACKSPACE => {
                self.text.pop();
            }
            _ => {
                self.text.push(char);
                let bounds = self.text_box.bounds();
                let draw_state = self.text_box.draw_state();
                let text_draw_state = draw_state.downcast_ref::<TextState>().unwrap();
                if !text_draw_state.text_fits(&self.text, bounds) {
                    self.text.pop();
                }
            }
        }
        self.update_text();
        args.widget.event(TextUpdated(self.text.clone()));
    }

    fn text_updated(&mut self, event: &TextUpdated, _: EventArgs) {
        let &TextUpdated(ref text) = event;
        self.text = text.clone();
        self.update_text();
    }

    fn style_updated(&mut self, _: &StyleUpdated, _: EventArgs) {
        self.update_text();
    }
}

component_style!{pub struct EditText<name="scroll", style=EditTextStyle> {
    rect: RectStyle = style!(RectStyle {
        border: Some((1.0, GRAY_70)),
        corner_radius: Some(3.0),
    }),
    focused_rect: Option<RectStyle> = Some(style!(RectStyle {
        border: Some((1.0, BLUE)),
    })),
}}

impl WidgetModifier for EditText {
    fn apply(&self, widget: &mut Widget) {
        let mut text_widget = Widget::new("edit_text_text");
        let mut draw_style = DrawStyle::from(self.rect.clone());
        widget
            .add_handler(|_: &WidgetAttachedEvent, args: EventArgs| {
                args.ui.event(KeyboardInputEvent::AddFocusable(args.widget));
            })
            .add_handler(|_: &WidgetDetachedEvent, args: EventArgs| {
                args.ui.event(KeyboardInputEvent::RemoveFocusable(args.widget));
            })
            .add_handler(EditTextHandler {
                text_box: text_widget.clone(),
                text: "".to_owned(),
            })
            .make_focusable();
        EditTextHandler::add_adapters(widget);

        if let Some(ref focused_rect) = self.focused_rect {
            draw_style.prop_style(FOCUSED.clone(), focused_rect.clone());
        }
        widget.set_draw_style(draw_style);

        text_widget
            .set_draw_style(DrawStyle::from(TextStyle::default()))
            .add_handler(TextHeightHandler::default());

        text_widget.layout().add(constraints![
            align_left(widget).padding(5.0),
            align_top(widget).padding(5.0),
            bound_by(widget).padding(5.0),
        ]);
        widget.add_child(text_widget);
    }
}

// Ensures the edit text is at least tall enough to fit the text. Width is unconstrained.
#[derive(Default)]
struct TextHeightHandler {
    measured_height: f32,
    size_constraints: Vec<Constraint>,
}

impl EventHandler<StyleUpdated> for TextHeightHandler {
    fn handle(&mut self, _: &StyleUpdated, mut args: EventArgs) {
        let line_height = {
            let draw_state = args.widget.draw_state();
            let text_draw_state = draw_state.downcast_ref::<TextState>().unwrap();
            text_draw_state.line_height()
        };
        if self.measured_height != line_height {
            let size_constraints = min_height(line_height).build(&args.widget.layout_vars());
            let mut layout = args.widget.layout();
            for constraint in self.size_constraints.drain(..) {
                layout.remove_constraint(constraint);
            }
            layout.add(size_constraints.clone());
            self.size_constraints = size_constraints;
            self.measured_height = line_height;
        }
    }
}
