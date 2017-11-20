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
use widget::property::states::*;

const BACKSPACE: char = '\u{8}';

#[derive(Debug)]
enum EditTextEvent {
    WidgetReceivedCharacter(char),
    TextUpdated(String),
    StyleUpdated,
}

#[derive(Default)]
struct EditTextHandler {
    text: String,
}

impl EventHandler<EditTextEvent> for EditTextHandler {
    fn handle(&mut self, event: &EditTextEvent, mut args: EventArgs) {
        match *event {
            EditTextEvent::WidgetReceivedCharacter(char) => {
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
                args.widget.event(TextUpdated(text.clone()));
            },
            EditTextEvent::StyleUpdated => {
                args.widget.update(|state: &mut TextState| {
                    state.text = self.text.clone()
                });
            },
            EditTextEvent::TextUpdated(ref text) => {
                self.text = text.clone();
                args.widget.update(|state: &mut TextState| {
                    state.text = text.clone()
                });
            }
        }
    }
}

pub fn text_change_handle(event: &TextUpdated, mut args: EventArgs) {
    args.widget.update(|state: &mut TextState| state.text = event.0.clone());
}

pub struct TextUpdated(pub String);


pub struct EditTextBuilder {
    pub widget: WidgetBuilder,
    pub text_widget: WidgetBuilder,
}

impl Default for EditTextBuilder {
    fn default() -> Self {
        let default_border = Some((1.0, GRAY_70));
        let rect_style = RectComponentStyle {
            border: Some(default_border),
            corner_radius: Some(Some(3.0)),
            ..RectComponentStyle::default()
        };
        let mut widget = WidgetBuilder::new("edit_text");
        widget
            .set_draw_style(rect_style)
            .add_handler(|_: &WidgetAttachedEvent, args: EventArgs| {
                args.ui.event(KeyboardInputEvent::AddFocusable(args.widget));
            })
            .add_handler(|_: &WidgetDetachedEvent, args: EventArgs| {
                args.ui.event(KeyboardInputEvent::RemoveFocusable(args.widget));
            })
            .make_focusable();

        widget.set_draw_style_prop(FOCUSED.clone(), RectComponentStyle {
            border: Some(Some((1.0, BLUE))),
            ..RectComponentStyle::default()
        });

        let mut text_widget = WidgetBuilder::new("edit_text_text");
        text_widget
            .set_draw_style(TextComponentStyle::default())
            .add_handler(TextUpdatedHandler::default())
            .add_handler(|event: &WidgetReceivedCharacter, args: EventArgs| args.widget.event(EditTextEvent::WidgetReceivedCharacter(event.0)))
            .add_handler(|event: &TextUpdated, args: EventArgs| args.widget.event(EditTextEvent::TextUpdated(event.0.clone())))
            .add_handler(|_: &StyleUpdated, args: EventArgs| args.widget.event(EditTextEvent::StyleUpdated))
            .add_handler(EditTextHandler::default());

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
        self.text_widget.add_handler(callback);
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
