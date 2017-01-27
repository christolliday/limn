use widget::{EventArgs, EventHandler, WidgetProperty, ChangePropEvent, WidgetNotifyEvent};
use widgets::primitives::RectDrawable;
use event::{self, EventId, EventAddress, Signal};
use resources::Id;
use color::*;

const WIDGET_LIST_ITEM_SELECTED: EventId = EventId("WIDGET_LIST_ITEM_SELECTED");

pub struct ListHandler {
    selected: Option<Id>,
}
impl ListHandler {
    pub fn new() -> Self {
        ListHandler { selected: None }
    }
}
impl EventHandler for ListHandler {
    fn event_id(&self) -> EventId {
        WIDGET_LIST_ITEM_SELECTED
    }
    fn handle_event(&mut self, mut args: EventArgs) {
        let selected = args.event.data::<Id>();
        if let Some(old_selected) = self.selected {
            if selected != &old_selected {
                let event = ChangePropEvent::new(WidgetProperty::Selected, false);
                args.event_queue.push(EventAddress::SubTree(old_selected), Box::new(event));
            }
        }
        self.selected = Some(*selected);
    }
}

pub struct ListItemHandler {
    list_id: Id,
}
impl ListItemHandler {
    pub fn new(list_id: Id) -> Self {
        ListItemHandler { list_id: list_id }
    }
}
impl EventHandler for ListItemHandler {
    fn event_id(&self) -> EventId {
        event::WIDGET_PRESS
    }
    fn handle_event(&mut self, mut args: EventArgs) {
        if !args.props.contains(&WidgetProperty::Selected) {
            let event = ChangePropEvent::new(WidgetProperty::Selected, true);
            args.event_queue.push(EventAddress::SubTree(args.widget_id), Box::new(event));
            let event = WidgetNotifyEvent::new(WIDGET_LIST_ITEM_SELECTED, args.widget_id);
            args.event_queue.push(EventAddress::Widget(self.list_id), Box::new(event));
        }
    }
}

pub struct ListItemPropsHandler {}
impl EventHandler for ListItemPropsHandler {
    fn event_id(&self) -> EventId {
        event::WIDGET_PROPS_CHANGED
    }
    fn handle_event(&mut self, mut args: EventArgs) {
        let selected = args.props.contains(&WidgetProperty::Selected);
        let hover = args.props.contains(&WidgetProperty::Hover);
        let color_selected = BLUE;
        let color_hover = [0.6, 0.6, 0.6, 1.0];
        let color_none = [0.3, 0.3, 0.3, 1.0];
        let color =
            if selected { color_selected }
            else if hover { color_hover }
            else { color_none };
        args.state.update(|state: &mut RectDrawable| state.background = color);
    }
}