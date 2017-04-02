use std::collections::HashMap;

use stable_bst::map::TreeMap;
use stable_bst::Bound::{Excluded, Unbounded};

use widget::{WidgetBuilder, WidgetBuilderCore};
use widget::property::{PropChange, Property};
use resources::WidgetId;
use input::mouse::ClickEvent;
use event::{Target, Queue, UiEventHandler, UiEventArgs};
use app::App;

use glutin;

#[derive(Clone, Debug)]
pub struct ReceivedCharacter(pub char);
#[derive(Clone, Debug)]
pub struct KeyboardInput(pub glutin::ElementState, pub glutin::ScanCode, pub Option<glutin::VirtualKeyCode>);
#[derive(Debug)]
pub struct WidgetKeyboardInput(pub glutin::ElementState, pub glutin::ScanCode, pub Option<glutin::VirtualKeyCode>);
#[derive(Debug)]
pub struct WidgetReceivedCharacter(pub char);

/**
Note on focus:
The tab key iterates through the widgets that have registered as focusable.
Currently the order of this iteration is just based on the order the widgets
are registered as focusable.
Later on maybe it should be based on the relative positioning of widgets (could get
ugly updating the treemap as widgets change position), or some user defined ordering.
*/
pub struct FocusHandler {
    focusable_map: HashMap<WidgetId, usize>,
     // can replace TreeMap with std BTreeMap once the range API or similar is stable
    focusable: TreeMap<usize, WidgetId>,
    focused: Option<WidgetId>,
    focus_index_max: usize,
}
impl FocusHandler {
    pub fn new() -> Self {
        FocusHandler {
            focusable_map: HashMap::new(),
            focusable: TreeMap::new(),
            focused: None,
            focus_index_max: 0,
        }
    }
    fn set_focus(&mut self, new_focus: Option<WidgetId>, queue: &mut Queue) {
        if new_focus != self.focused {
            if let Some(focused) = self.focused {
                queue.push(Target::SubTree(focused), PropChange::Remove(Property::Focused));
            }
            self.focused = new_focus;
            if let Some(focused) = self.focused {
                queue.push(Target::SubTree(focused), PropChange::Add(Property::Focused));
            }
        }
    }
}
impl UiEventHandler<KeyboardInputEvent> for FocusHandler {
    fn handle(&mut self, event: &KeyboardInputEvent, mut args: UiEventArgs) {
        match event {
            &KeyboardInputEvent::AddFocusable(widget_id) => {
                self.focusable.insert(self.focus_index_max, widget_id);
                self.focusable_map.insert(widget_id, self.focus_index_max);
                self.focus_index_max += 1;
                if self.focused.is_none() {
                    self.set_focus(Some(widget_id), args.queue);
                }
            }
            &KeyboardInputEvent::RemoveFocusable(widget_id) => {
                if let Some(focused) = self.focused {
                    if focused == widget_id {
                        self.set_focus(None, args.queue);
                    }
                }
                let index = self.focusable_map.remove(&widget_id).unwrap();
                self.focusable.remove(&index);
            }
            &KeyboardInputEvent::FocusChange(new_focus) => {
                self.set_focus(new_focus, args.queue);
            }
            &KeyboardInputEvent::KeyboardInput(ref key_input) => {
                if let Some(focused) = self.focused {
                    let &KeyboardInput(state, scan_code, maybe_keycode) = key_input;
                    let event = WidgetKeyboardInput(state, scan_code, maybe_keycode);
                    args.queue.push(Target::SubTree(focused), event);
                }
            }
            &KeyboardInputEvent::ReceivedCharacter(ref received_char) => {
                let &ReceivedCharacter(char) = received_char;
                if char == '\t' {
                    let mut new_focus = self.focused.and_then(|focused| {
                        let index = self.focusable_map.get(&focused).unwrap();
                        self.focusable.range(Excluded(index), Unbounded).next().map(|(_, v)| v.clone())
                    });
                    if new_focus.is_none() {
                        // focus on first, if any
                        new_focus = self.focusable.iter().next().map(|(_, v)| v.clone());
                    }
                    self.set_focus(new_focus, args.queue);
                } else if let Some(focused) = self.focused {
                    let event = WidgetReceivedCharacter(char);
                    args.queue.push(Target::SubTree(focused), event);
                }
            }
        }
    }
}

pub enum KeyboardInputEvent {
    AddFocusable(WidgetId),
    RemoveFocusable(WidgetId),
    FocusChange(Option<WidgetId>),
    KeyboardInput(KeyboardInput),
    ReceivedCharacter(ReceivedCharacter),
}

impl WidgetBuilder {
    pub fn make_focusable(&mut self) -> &mut Self {
        self.add_handler_fn(|_: &ClickEvent, args| {
            args.queue.push(Target::Ui, KeyboardInputEvent::FocusChange(Some(args.widget.id)));
        })
    }
}

impl App {
    pub fn add_keyboard_handlers(&mut self) {
        self.add_handler_fn(|event: &KeyboardInput, args| {
            args.queue.push(Target::Ui, KeyboardInputEvent::KeyboardInput(event.clone()));
        });
        self.add_handler_fn(|event: &ReceivedCharacter, args| {
            args.queue.push(Target::Ui, KeyboardInputEvent::ReceivedCharacter(event.clone()));
        });
        self.add_handler(FocusHandler::new());
    }
}