use widget::{EventArgs, EventHandler};
use widget::property::Property;

pub struct FocusChangeEvent(Option<WidgetId>);

pub struct FocusHandler {
    focused: Option<WidgetId>,
}
impl EventHandler<FocusChangeEvent> for FocusHandler {
    fn handle(&mut self, event: &FocusChangeEvent, mut args: EventArgs) {
        self.focused = event.0;
    }
}

pub struct WidgetFocusHandler;
impl EventHandler<ClickEvent> for WidgetFocusHandler {
    fn handle(&mut self, event: &ClickEvent, mut args: EventArgs) {
        args.event_queue.push(EventAddress::Ui, FocusChangeEvent(args.widget_id));
    }
}
