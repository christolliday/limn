use cassowary::strength::*;

use widget::builder::WidgetBuilder;
use widget::style::Value;
use widget::{EventHandler, EventArgs, HandlerWrapper};
use event::Target;
use drawable::rect::{RectDrawable, RectStyleField};
use widgets::drag::{DragEvent, WidgetDrag};
use resources::WidgetId;
use util::Dimensions;

pub struct SliderBuilder {
    pub widget: WidgetBuilder,
}
impl SliderBuilder {
    pub fn new() -> Self {
        let rect_color = [0.1, 0.1, 0.1, 1.0];
        let style = vec![RectStyleField::BackgroundColor(Value::Single(rect_color))];
        let mut widget = WidgetBuilder::new()
            .set_drawable_with_style(RectDrawable::new(), style);
        widget.layout.dimensions(Dimensions {
            width: 200.0,
            height: 30.0,
        });

        let rect_color = [0.4, 0.4, 0.4, 1.0];
        let style = vec![RectStyleField::BackgroundColor(Value::Single(rect_color))];
        let mut slider_handle = WidgetBuilder::new()
            .set_drawable_with_style(RectDrawable::new(), style)
            .draggable()
            .add_handler(DragHandler::new(widget.id));
        slider_handle.layout.dimensions(Dimensions {
            width: 30.0,
            height: 30.0,
        });

        widget.add_child(slider_handle);
        SliderBuilder { widget: widget }
    }
    // tmp, should be able to use inner widget method
    pub fn add_handler<E: 'static, T: EventHandler<E> + 'static>(mut self, handler: T) -> Self {
        self.widget.event_handlers.push(HandlerWrapper::new(handler));
        self
    }
    pub fn on_val_changed<F>(self, on_val_changed: F) -> Self
        where F: Fn(f64) + 'static
    {
        self.add_handler(SliderHandler::new(on_val_changed))
    }
}

pub struct SliderHandler<F: Fn(f64)> {
    callback: F,
}
impl<F: Fn(f64)> SliderHandler<F> {
    pub fn new(callback: F) -> Self {
        SliderHandler { callback: callback }
    }
}
impl<F> EventHandler<MovedSliderWidgetEvent> for SliderHandler<F>
    where F: Fn(f64) {
    fn handle(&mut self, event: &MovedSliderWidgetEvent, mut args: EventArgs) {
        let range = args.widget.layout.bounds.width - (event.slider_right - event.slider_left);
        let val = (event.slider_left - args.widget.layout.bounds.left) / range;
        (self.callback)(val);
        args.event_state.handled = true;
    }
}

struct MovedSliderWidgetEvent {
    slider_left: f64,
    slider_right: f64,
}
struct DragHandler {
    container: WidgetId,
    start_pos: f64,
}
impl DragHandler {
    pub fn new(container: WidgetId) -> Self {
        DragHandler { container: container, start_pos: 0.0 }
    }
}
impl EventHandler<WidgetDrag> for DragHandler {
    fn handle(&mut self, event: &WidgetDrag, args: EventArgs) {
        let EventArgs { solver, widget, .. } = args;
        let ref layout = widget.layout;
        let &WidgetDrag { ref drag_type, position } = event;
        let drag_pos = position.x;
        match *drag_type {
            DragEvent::DragStart => {
                self.start_pos = drag_pos - solver.get_value(layout.left);
            }
            _ => {
                solver.update_solver(|solver| {
                    if !solver.has_edit_variable(&layout.left) {
                        solver.add_edit_variable(layout.left, STRONG).unwrap();
                    }
                    solver.suggest_value(layout.left, drag_pos - self.start_pos).unwrap();
                });
                let event = MovedSliderWidgetEvent { slider_left: layout.bounds.left, slider_right: layout.bounds.right() };
                args.queue.push(Target::Widget(self.container), event);
            }
        }
    }
}
