use input::mouse::ClickEvent;
use event::{Target, WidgetEventHandler, WidgetEventArgs};
use widget::{WidgetBuilder, WidgetBuilderCore, BuildWidget};
use widget::property::Property;
use widget::property::states::*;
use widgets::drag::{DragEvent, WidgetDrag};
use layout::constraint::*;
use drawable::rect::{RectDrawable, RectStyleable};
use drawable::ellipse::{EllipseDrawable, EllipseStyleable};
use resources::WidgetId;
use ui::ChildAttachedEvent;
use util::{Size, RectExt};

pub struct SliderBuilder {
    pub widget: WidgetBuilder,
}
widget_builder!(SliderBuilder);

impl SliderBuilder {
    pub fn new() -> Self {
        let mut slider = WidgetBuilder::new();

        let light_gray = [0.8, 0.8, 0.8, 1.0];
        let dark_gray = [0.7, 0.7, 0.7, 1.0];
        let border_color = [0.3, 0.3, 0.3, 1.0];
        let blue = [0.4, 0.4, 0.9, 1.0];
        let style = style!(
            EllipseStyleable::BackgroundColor: light_gray,
            EllipseStyleable::Border: Some((1.0, border_color))
        );
        let mut slider_handle = WidgetBuilder::new();
        slider_handle
            .set_drawable_with_style(EllipseDrawable::new(), style)
            .add_handler_fn(|event: &WidgetDrag, args| {
                let event = SliderHandleInput::WidgetDrag(event.clone());
                event!(Target::Widget(args.widget.id), event);
            })
            .add_handler(DragHandler::new(slider.id()))
            .make_draggable();
        layout!(slider_handle: dimensions(Size::new(30.0, 30.0)));

        let mut slider_bar_left = WidgetBuilder::new();
        let bar_style = style!(
            RectStyleable::Border: Some((0.5, border_color)),
            RectStyleable::CornerRadius: Some(3.0));
        let style = style!(parent: bar_style, RectStyleable::BackgroundColor:
            selector!(blue, INACTIVE: light_gray));
        slider_bar_left.set_drawable_with_style(RectDrawable::new(), style);

        layout!(slider_bar_left:
            height(10.0),
            center_vertical(&slider),
            align_left(&slider).padding(15.0),
            to_left_of(&slider_handle).padding(-10.0));

        let mut slider_bar_right = WidgetBuilder::new();
        let style = style!(parent: bar_style, RectStyleable::BackgroundColor: dark_gray);
        slider_bar_right.set_drawable_with_style(RectDrawable::new(), style);
        layout!(slider_bar_right:
            height(10.0),
            center_vertical(&slider),
            align_right(&slider).padding(15.0),
            to_right_of(&slider_handle).padding(-10.0));

        let handle_id = slider_handle.id();
        slider.add_handler_fn(move |event: &SetSliderValue, args| {
            let bounds = args.widget.bounds;
            let event = SliderHandleInput::SetValue((event.0, bounds.width(), bounds.left()));
            event!(Target::Widget(handle_id), event);
        });
        slider.add_handler_fn(move |event: &ClickEvent, args| {
            let bounds = args.widget.bounds;
            let event = SliderHandleInput::SliderClicked((event.position.x, bounds.width(), bounds.left()));
            event!(Target::Widget(handle_id), event);
        });

        slider.add_child(slider_bar_left);
        slider.add_child(slider_bar_right);
        slider.add_child(slider_handle);
        SliderBuilder { widget: slider }
    }
    pub fn set_value(&mut self, value: f64) -> &mut Self {
        // need to update position after widget is created because the slider position
        // depends on the measured width of the slider and slider handle
        self.add_handler_fn(move |_: &ChildAttachedEvent, args| {
            event!(Target::Widget(args.widget.id), SetSliderValue(value));
        });
        self
    }
    pub fn on_value_changed<F>(&mut self, on_value_changed: F) -> &mut Self
        where F: Fn(f64, &mut WidgetEventArgs) + 'static
    {
        self.widget.add_handler_fn(move |event: &MovedSliderWidgetEvent, mut args| {
            let bounds = args.widget.bounds;
            let range = bounds.width() - (event.slider_right - event.slider_left);
            let val = (event.slider_left - bounds.left()) / range;
            on_value_changed(val, &mut args);
            *args.handled = true;
        });
        self
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
    SliderClicked((f64, f64, f64)),
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
        let bounds = args.widget.bounds;
        match *event {
            SliderHandleInput::WidgetDrag(ref event) => {
                if args.widget.props.contains(&Property::Inactive) {
                    return;
                }
                let &WidgetDrag { ref drag_type, position } = event;
                let drag_pos = position.x;
                match *drag_type {
                    DragEvent::DragStart => {
                        self.start_pos = drag_pos - bounds.left();
                    }
                    _ => {
                        args.widget.update_layout(|layout| {
                            layout.edit_left().set(drag_pos - self.start_pos);
                        }, args.solver);
                        let event = MovedSliderWidgetEvent { slider_left: bounds.left(), slider_right: bounds.right() };
                        event!(Target::Widget(self.container), event);
                    }
                }
            }
            SliderHandleInput::SetValue((value, parent_width, parent_left)) => {
                let pos = value * (parent_width - bounds.width());
                args.widget.update_layout(|layout| {
                    layout.edit_left().set(parent_left + pos);
                }, args.solver);
            }
            SliderHandleInput::SliderClicked((position, parent_width, parent_left)) => {
                if args.widget.props.contains(&Property::Inactive) {
                    return;
                }
                let handle_radius = bounds.width() / 2.0;
                let min = parent_left + handle_radius;
                let max = parent_left + parent_width - handle_radius;
                let position = f64::min(f64::max(position, min), max);
                args.widget.update_layout(|layout| {
                    layout.edit_left().set(position - handle_radius);
                }, args.solver);
                let event = MovedSliderWidgetEvent { slider_left: position - handle_radius, slider_right: position + handle_radius };
                event!(Target::Widget(self.container), event);
            }
        }
    }
}
