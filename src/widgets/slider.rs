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
use util::{Rect, RectExt};

#[derive(Clone, Copy)]
pub enum Orientation {
    Horizontal,
    Vertical,
}
pub struct SliderBuilder {
    pub widget: WidgetBuilder,
}
widget_builder!(SliderBuilder);

impl SliderBuilder {
    pub fn new(orientation: Orientation) -> Self {
        let mut slider = WidgetBuilder::new();
        slider.set_debug_name("slider");

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
            .set_debug_name("slider_handle")
            .set_drawable_with_style(EllipseDrawable::new(), style)
            .add_handler_fn(|event: &WidgetDrag, args| {
                let event = SliderHandleInput::WidgetDrag(event.clone());
                event!(Target::Widget(args.widget.id), event);
            })
            .add_handler(DragHandler::new(orientation, slider.id()))
            .make_draggable();

        let mut slider_bar_pre = WidgetBuilder::new();
        let bar_style = style!(
            RectStyleable::Border: Some((0.5, border_color)),
            RectStyleable::CornerRadius: Some(3.0));
        let style = style!(parent: bar_style, RectStyleable::BackgroundColor:
            selector!(blue, INACTIVE: light_gray));
        slider_bar_pre
            .set_debug_name("slider_bar_pre")
            .set_drawable_with_style(RectDrawable::new(), style);

        let mut slider_bar_post = WidgetBuilder::new();
        let style = style!(parent: bar_style, RectStyleable::BackgroundColor: dark_gray);
        slider_bar_post
            .set_debug_name("slider_bar_post")
            .set_drawable_with_style(RectDrawable::new(), style);

        layout!(slider_handle: aspect_ratio(1.0));
        match orientation {
            Orientation::Horizontal => {
                layout!(slider:
                    height(30.0));
                layout!(slider_bar_pre:
                    height(10.0),
                    center_vertical(&slider),
                    align_left(&slider).padding(15.0),
                    to_left_of(&slider_handle).padding(-10.0));
                layout!(slider_bar_post:
                    height(10.0),
                    center_vertical(&slider),
                    align_right(&slider).padding(15.0),
                    to_right_of(&slider_handle).padding(-10.0));
                layout!(slider_handle:
                    match_height(&slider));
            }
            Orientation::Vertical => {
                layout!(slider:
                    width(30.0));
                layout!(slider_bar_pre:
                    width(10.0),
                    center_horizontal(&slider),
                    align_top(&slider).padding(15.0),
                    above(&slider_handle).padding(-10.0));
                layout!(slider_bar_post:
                    width(10.0),
                    center_horizontal(&slider),
                    align_bottom(&slider).padding(15.0),
                    below(&slider_handle).padding(-10.0));
                layout!(slider_handle:
                    match_width(&slider));
            }
        }


        let handle_id = slider_handle.id();
        slider.add_handler_fn(move |event: &SetSliderValue, args| {
            let bounds = args.widget.bounds;
            let event = SliderHandleInput::SetValue((event.0, bounds));
            event!(Target::Widget(handle_id), event);
        });
        slider.add_handler_fn(move |event: &ClickEvent, args| {
            let bounds = args.widget.bounds;
            let position = if let Orientation::Horizontal = orientation {
                event.position.x
            } else {
                event.position.y
            };
            let event = SliderHandleInput::SliderClicked((position, bounds));
            event!(Target::Widget(handle_id), event);
        });

        slider.add_child(slider_bar_pre);
        slider.add_child(slider_bar_post);
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
            let (slider_start, slider_size) = if let Orientation::Horizontal = event.orientation {
                (bounds.left(), bounds.width())
            } else {
                (bounds.top(), bounds.height())
            };
            let range = slider_size - (event.handle_size * 2.0);
            let val = (event.slider_pos - event.handle_size - slider_start) / range;
            on_value_changed(val, &mut args);
            *args.handled = true;
        });
        self
    }
}

struct MovedSliderWidgetEvent {
    orientation: Orientation,
    slider_pos: f64,
    handle_size: f64,
}

pub struct SetSliderValue(pub f64);
pub enum SliderHandleInput {
    WidgetDrag(WidgetDrag),
    SetValue((f64, Rect)),
    SliderClicked((f64, Rect)),
}
struct DragHandler {
    orientation: Orientation,
    container: WidgetId,
    start_pos: f64,
}
impl DragHandler {
    pub fn new(orientation: Orientation, container: WidgetId) -> Self {
        DragHandler {
            orientation: orientation,
            container: container,
            start_pos: 0.0
        }
    }
}
impl WidgetEventHandler<SliderHandleInput> for DragHandler {
    fn handle(&mut self, event: &SliderHandleInput, args: WidgetEventArgs) {
        let bounds = args.widget.bounds;
        let handle_radius = if let Orientation::Horizontal = self.orientation {
            bounds.width() / 2.0
        } else {
            bounds.height() / 2.0
        };
        match *event {
            SliderHandleInput::WidgetDrag(ref event) => {
                if args.widget.props.contains(&Property::Inactive) {
                    return;
                }
                let &WidgetDrag { ref drag_type, position } = event;
                let (drag_pos, bounds_start, bounds_end) = if let Orientation::Horizontal = self.orientation {
                    (position.x, bounds.left(), bounds.right())
                } else {
                    (position.y, bounds.top(), bounds.bottom())
                };
                match *drag_type {
                    DragEvent::DragStart => {
                        self.start_pos = drag_pos - bounds_start;
                    }
                    _ => {
                        let drag_to = drag_pos - self.start_pos;
                        args.widget.update_layout(|layout| {
                            if let Orientation::Horizontal = self.orientation {
                                layout.edit_left().set(drag_to);
                            } else {
                                layout.edit_top().set(drag_to);
                            }
                        }, args.solver);
                        let event = MovedSliderWidgetEvent {
                            orientation: self.orientation,
                            slider_pos: (bounds_start + bounds_end) / 2.0,
                            handle_size: handle_radius
                        };
                        event!(Target::Widget(self.container), event);
                    }
                }
            }
            SliderHandleInput::SetValue((value, parent_bounds)) => {
                let pos = if let Orientation::Horizontal = self.orientation {
                    parent_bounds.left() + value * (parent_bounds.width() - bounds.width())
                } else {
                    parent_bounds.top() + value * (parent_bounds.height() - bounds.height())
                };
                args.widget.update_layout(|layout| {
                    layout.edit_left().set(pos);
                }, args.solver);
            }
            SliderHandleInput::SliderClicked((position, parent_bounds)) => {
                if args.widget.props.contains(&Property::Inactive) {
                    return;
                }
                let position = if let Orientation::Horizontal = self.orientation {
                    let min = parent_bounds.left() + handle_radius;
                    let max = parent_bounds.left() + parent_bounds.width() - handle_radius;
                    let position = f64::min(f64::max(position, min), max);
                    args.widget.update_layout(|layout| {
                        layout.edit_left().set(position - handle_radius);
                    }, args.solver);
                    position
                } else {
                    let min = parent_bounds.top() + handle_radius;
                    let max = parent_bounds.top() + parent_bounds.height() - handle_radius;
                    let position = f64::min(f64::max(position, min), max);
                    args.widget.update_layout(|layout| {
                        layout.edit_top().set(position - handle_radius);
                    }, args.solver);
                    position
                };
                let event = MovedSliderWidgetEvent {
                    orientation: self.orientation,
                    slider_pos: position,
                    handle_size: handle_radius
                };
                event!(Target::Widget(self.container), event);
            }
        }
    }
}
