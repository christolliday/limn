use cassowary::strength::*;

use event::{Target, WidgetEventHandler, WidgetEventArgs};
use widget::{WidgetBuilder, WidgetBuilderCore, BuildWidget};
use widget::property::Property;
use widgets::drag::{DragEvent, WidgetDrag};
use drawable::rect::{RectDrawable, RectStyleField};
use resources::WidgetId;
use util::Dimensions;

pub struct SliderBuilder {
    pub widget: WidgetBuilder,
}
impl SliderBuilder {
    pub fn new() -> Self {
        let style = style!(RectStyleField::BackgroundColor: [0.1, 0.1, 0.1, 1.0]);
        let mut widget = WidgetBuilder::new();
        widget.set_drawable_with_style(RectDrawable::new(), style);
        widget.layout().dimensions(Dimensions {
            width: 200.0,
            height: 30.0,
        });

        let style = style!(RectStyleField::BackgroundColor: [0.4, 0.4, 0.4, 1.0]);
        let mut slider_handle = WidgetBuilder::new();
        slider_handle
            .set_drawable_with_style(RectDrawable::new(), style)
            .add_handler_fn(|event: &WidgetDrag, args| {
                let event = SliderHandleInput::WidgetDrag(event.clone());
                args.queue.push(Target::Widget(args.widget.id), event);
            })
            .add_handler(DragHandler::new(widget.id()))
            .make_draggable();
        slider_handle.layout().dimensions(Dimensions {
            width: 30.0,
            height: 30.0,
        });

        let handle_id = slider_handle.id();
        widget.add_handler_fn(move |event: &SetSliderValue, args| {
            let bounds = args.widget.layout.bounds();
            let event = SliderHandleInput::SetValue((event.0, bounds.width, bounds.left));
            args.queue.push(Target::Widget(handle_id), event);
        });

        widget.add_child(slider_handle);
        SliderBuilder { widget: widget }
    }
    pub fn on_val_changed<F>(&mut self, on_val_changed: F) -> &mut Self
        where F: Fn(f64, &mut WidgetEventArgs) + 'static
    {
        self.widget.add_handler_fn(move |event: &MovedSliderWidgetEvent, mut args| {
            let bounds = args.widget.layout.bounds();
            let range = bounds.width - (event.slider_right - event.slider_left);
            let val = (event.slider_left - bounds.left) / range;
            on_val_changed(val, &mut args);
            *args.handled = true;
        });
        self
    }
}

impl AsMut<WidgetBuilder> for SliderBuilder {
    fn as_mut(&mut self) -> &mut WidgetBuilder {
        &mut self.widget
    }
}
impl BuildWidget for SliderBuilder {
    fn build(self) -> WidgetBuilder {
        self.widget
    }
}

struct MovedSliderWidgetEvent {
    slider_left: f64,
    slider_right: f64,
}

pub struct SetSliderValue(pub f64);
pub enum SliderHandleInput {
    WidgetDrag(WidgetDrag),
    SetValue((f64, f64, f64)),
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
impl WidgetEventHandler<SliderHandleInput> for DragHandler {
    fn handle(&mut self, event: &SliderHandleInput, args: WidgetEventArgs) {

        let ref layout = args.widget.layout;
        let bounds = layout.bounds();

        match *event {
            SliderHandleInput::WidgetDrag(ref event) => {
                if args.widget.props.contains(&Property::Inactive) {
                    return;
                }
                let &WidgetDrag { ref drag_type, position } = event;
                let drag_pos = position.x;
                match *drag_type {
                    DragEvent::DragStart => {
                        self.start_pos = drag_pos - bounds.left;
                    }
                    _ => {
                        args.solver.update_solver(|solver| {
                            if !solver.has_edit_variable(&layout.left) {
                                solver.add_edit_variable(layout.left, STRONG).unwrap();
                            }
                            solver.suggest_value(layout.left, drag_pos - self.start_pos).unwrap();
                        });
                        let event = MovedSliderWidgetEvent { slider_left: bounds.left, slider_right: bounds.right() };
                        args.queue.push(Target::Widget(self.container), event);
                    }
                }
            }
            SliderHandleInput::SetValue((value, parent_width, parent_left)) => {
                args.solver.update_solver(|solver| {
                    let pos = value * (parent_width - bounds.width);
                    if !solver.has_edit_variable(&layout.left) {
                        solver.add_edit_variable(layout.left, STRONG).unwrap();
                    }
                    solver.suggest_value(layout.left, parent_left + pos).unwrap();
                });
            }
        }
    }
}
