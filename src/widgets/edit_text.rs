use cassowary::Constraint;

use layout::constraint::ConstraintBuilder;
use layout::constraint::*;
use widget::{WidgetBuilder, StyleUpdated};
use ui::{WidgetAttachedEvent, WidgetDetachedEvent};
use input::keyboard::{WidgetReceivedCharacter, KeyboardInputEvent};
use draw::rect::RectComponentStyle;
use draw::text::{TextState, TextComponentStyle};
use event::{EventHandler, EventArgs};
use color::*;
use widget::WidgetRef;
use widget::property::states::*;
use style::WidgetModifier;

const BACKSPACE: char = '\u{8}';

#[derive(Debug)]
enum EditTextEvent {
    WidgetReceivedCharacter(char),
    TextUpdated(String),
    StyleUpdated,
}

struct EditTextHandler {
    text_box: WidgetRef,
    text: String,
}

impl EditTextHandler {
    fn update_text(&mut self) {
        let text = self.text.clone();
        self.text_box.update(|state: &mut TextState| {
            state.text = text;
        });
    }
}
impl EventHandler<EditTextEvent> for EditTextHandler {
    fn handle(&mut self, event: &EditTextEvent, args: EventArgs) {
        match *event {
            EditTextEvent::WidgetReceivedCharacter(char) => {
                let text = {
                    let bounds = self.text_box.bounds();
                    let draw_state = self.text_box.draw_state();
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
                self.text = text.clone();
                self.update_text();
                args.widget.event(TextUpdated(text.clone()));
            },
            EditTextEvent::StyleUpdated => {
                self.update_text();
            },
            EditTextEvent::TextUpdated(ref text) => {
                self.text = text.clone();
                self.update_text();
            }
        }
    }
}

pub fn text_change_handle(event: &TextUpdated, mut args: EventArgs) {
    args.widget.update(|state: &mut TextState| state.text = event.0.clone());
}

pub struct TextUpdated(pub String);


component_style!{pub struct EditTextBuilder<name="scroll", style=EditTextStyle> {
    rect: RectComponentStyle = RectComponentStyle {
        border: Some(Some((1.0, GRAY_70))),
        corner_radius: Some(Some(3.0)),
        ..RectComponentStyle::default()
    },
    focused_rect: Option<RectComponentStyle> = Some(RectComponentStyle {
        border: Some(Some((1.0, BLUE))),
        ..RectComponentStyle::default()
    }),
}}

impl WidgetModifier for EditTextBuilder {
    fn apply(&self, widget: &mut WidgetBuilder) {
        let mut text_widget = WidgetBuilder::new("edit_text_text");
        widget
            .set_draw_style(self.rect.clone())
            .add_handler(|_: &WidgetAttachedEvent, args: EventArgs| {
                args.ui.event(KeyboardInputEvent::AddFocusable(args.widget));
            })
            .add_handler(|_: &WidgetDetachedEvent, args: EventArgs| {
                args.ui.event(KeyboardInputEvent::RemoveFocusable(args.widget));
            })
            .add_handler(EditTextHandler {
                text_box: text_widget.widget_ref(),
                text: "".to_owned(),
            })
            .make_focusable();

        if let Some(ref focused_rect) = self.focused_rect {
            widget.set_draw_style_prop(FOCUSED.clone(), focused_rect.clone());
        }

        text_widget
            .set_draw_style(TextComponentStyle::default())
            .add_handler(TextUpdatedHandler::default());

        let widget_ref = widget.widget_ref();
        text_widget.add_handler(move |event: &WidgetReceivedCharacter, _: EventArgs| widget_ref.event(EditTextEvent::WidgetReceivedCharacter(event.0)));

        let widget_ref = widget.widget_ref();
        text_widget.add_handler(move |event: &TextUpdated, _: EventArgs| widget_ref.event(EditTextEvent::TextUpdated(event.0.clone())));

        let widget_ref = widget.widget_ref();
        text_widget.add_handler(move |_: &StyleUpdated, _: EventArgs| widget_ref.event(EditTextEvent::StyleUpdated));

        text_widget.layout().add(constraints![
            align_left(widget).padding(5.0),
            align_top(widget).padding(5.0),
            bound_by(widget).padding(5.0),
        ]);
        widget.add_child(text_widget);
    }
}

#[derive(Default)]
struct TextUpdatedHandler {
    measured_height: f32,
    size_constraints: Vec<Constraint>,
}

impl EventHandler<StyleUpdated> for TextUpdatedHandler {
    fn handle(&mut self, _: &StyleUpdated, mut args: EventArgs) {
        let line_height = {
            let draw_state = args.widget.draw_state();
            let text_draw_state = draw_state.downcast_ref::<TextState>().unwrap();
            text_draw_state.line_height()
        };
        if self.measured_height != line_height {
            let size_constraints = min_height(line_height).build(&args.widget.layout_vars());
            args.widget.update_layout(|layout| {
                for constraint in self.size_constraints.drain(..) {
                    layout.remove_constraint(constraint);
                }
                layout.add(size_constraints.clone())
            });
            self.size_constraints = size_constraints;
            self.measured_height = line_height;
        }
    }
}
