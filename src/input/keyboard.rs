//! Keyboard input handlers.

use std::collections::HashMap;

use stable_bst::map::TreeMap;
use stable_bst::Bound::{Excluded, Unbounded};

use widget::{WidgetRef, WidgetBuilder};
use widget::property::Property;
use input::mouse::ClickEvent;
use event::{EventHandler, EventArgs};
use app::App;

use glutin;

#[derive(Debug, Copy, Clone)]
pub struct ReceivedCharacter(pub char);
#[derive(Debug, Copy, Clone)]
pub struct KeyboardInput(pub glutin::KeyboardInput);
#[derive(Debug, Copy, Clone)]
pub struct WidgetKeyboardInput(pub glutin::KeyboardInput);
#[derive(Debug, Copy, Clone)]
pub struct WidgetReceivedCharacter(pub char);


/// Note on focus:
/// The tab key iterates through the widgets that have registered as focusable.
/// Currently the order of this iteration is just based on the order the widgets
/// are registered as focusable.
/// Later on maybe it should be based on the relative positioning of widgets (could get
/// ugly updating the treemap as widgets change position), or some user defined ordering.
#[derive(Default)]
pub struct FocusHandler {
    focusable_map: HashMap<WidgetRef, usize>,
    // can replace TreeMap with std BTreeMap once the range API or similar is stable
    focusable: TreeMap<usize, WidgetRef>,
    focused: Option<WidgetRef>,
    focus_index_max: usize,
}
impl FocusHandler {
    pub fn new() -> Self {
        Self::default()
    }
    fn set_focus(&mut self, new_focus: Option<WidgetRef>) {
        if new_focus != self.focused {
            if let Some(ref mut focused) = self.focused {
                focused.remove_prop(Property::Focused);
            }
            self.focused = new_focus;
            if let Some(ref mut focused) = self.focused {
                focused.add_prop(Property::Focused);
            }
        }
    }
}
impl EventHandler<KeyboardInputEvent> for FocusHandler {
    fn handle(&mut self, event: &KeyboardInputEvent, _: EventArgs) {
        match *event {
            KeyboardInputEvent::AddFocusable(ref widget_id) => {
                self.focusable.insert(self.focus_index_max, widget_id.clone());
                self.focusable_map.insert(widget_id.clone(), self.focus_index_max);
                self.focus_index_max += 1;
                if self.focused.is_none() {
                    self.set_focus(Some(widget_id.clone()));
                }
            }
            KeyboardInputEvent::RemoveFocusable(ref widget_id) => {
                if let Some(focused) = self.focused.clone() {
                    if focused == *widget_id {
                        self.set_focus(None);
                    }
                }
                let index = self.focusable_map.remove(widget_id).unwrap();
                self.focusable.remove(&index);
            }
            KeyboardInputEvent::FocusChange(ref new_focus) => {
                self.set_focus(new_focus.clone());
            }
            KeyboardInputEvent::KeyboardInput(ref key_input) => {
                if let Some(ref focused) = self.focused {
                    let &KeyboardInput(input) = key_input;
                    let event = WidgetKeyboardInput(input);
                    focused.event_subtree(event);
                }
            }
            KeyboardInputEvent::ReceivedCharacter(ref received_char) => {
                let &ReceivedCharacter(char) = received_char;
                if char == '\t' {
                    let mut new_focus = self.focused.clone().and_then(|focused| {
                        let index = &self.focusable_map[&focused];
                        self.focusable.range(Excluded(index), Unbounded).next().map(|(_, v)| v.clone())
                    });
                    if new_focus.is_none() {
                        // focus on first, if any
                        new_focus = self.focusable.iter().next().map(|(_, v)| v.clone());
                    }
                    self.set_focus(new_focus);
                } else if let Some(ref focused) = self.focused {
                    let event = WidgetReceivedCharacter(char);
                    focused.event_subtree(event);
                }
            }
        }
    }
}

pub enum KeyboardInputEvent {
    AddFocusable(WidgetRef),
    RemoveFocusable(WidgetRef),
    FocusChange(Option<WidgetRef>),
    KeyboardInput(KeyboardInput),
    ReceivedCharacter(ReceivedCharacter),
}

impl WidgetBuilder {
    pub fn make_focusable(&mut self) -> &mut Self {
        self.add_handler(|_: &ClickEvent, args: EventArgs| {
            args.ui.get_root().event(KeyboardInputEvent::FocusChange(Some(args.widget)));
        })
    }
}

impl App {
    pub fn add_keyboard_handlers(&mut self) {
        self.add_handler(|event: &KeyboardInput, args: EventArgs| {
            args.widget.event(KeyboardInputEvent::KeyboardInput(*event));
        });
        self.add_handler(|event: &ReceivedCharacter, args: EventArgs| {
            args.widget.event(KeyboardInputEvent::ReceivedCharacter(*event));
        });
        self.add_handler(FocusHandler::new());
    }
}
