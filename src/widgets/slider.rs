use cassowary::strength::*;

use input::mouse::ClickEvent;
use event::{Target, WidgetEventHandler, WidgetEventArgs};
use widget::{WidgetBuilder, WidgetBuilderCore, BuildWidget};
use widget::property::Property;
use widget::property::states::*;
use widgets::drag::{DragEvent, WidgetDrag};
use layout::constraint::*;
use layout::{LayoutRef, LayoutVars};
use drawable::rect::{RectDrawable, RectStyleable};
use drawable::ellipse::{EllipseDrawable, EllipseStyleable};
use resources::WidgetId;
use ui::ChildAttachedEvent;
use util::{Rect, RectExt, Scalar, Color};
use color::*;

#[derive(Clone, Copy)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

pub enum HandleStyle {
    Round,
    Square
}

pub enum BarStyle {
    NarrowRound,
    Wide,
}

pub struct SliderBuilder {
    pub widget: WidgetBuilder,
    pub slider_handle: WidgetBuilder,
    pub orientation: Orientation,
    pub init_value: f64,
    pub variable_handle_size: bool,
    pub handle_style: HandleStyle,
    pub bar_style: BarStyle,
    pub border: Option<(Scalar, Color)>,
    pub bar_color: Color,
    pub handle_color: Color,
    pub highlight: Option<Color>,
    pub width: Scalar,
}

impl SliderBuilder {
    pub fn new() -> Self {
        let mut slider = WidgetBuilder::new();
        slider.set_debug_name("slider");

        let mut slider_handle = WidgetBuilder::new();
        slider_handle
            .set_debug_name("slider_handle")
            .add_handler_fn(|event: &WidgetDrag, args| {
                let event = SliderHandleInput::WidgetDrag(event.clone());
                event!(Target::WidgetRef(args.widget), event);
            })
            .make_draggable();
        SliderBuilder {
            widget: slider,
            slider_handle: slider_handle,
            orientation: Orientation::Horizontal,
            init_value: 0.0,
            variable_handle_size: false,
            handle_style: HandleStyle::Round,
            bar_style: BarStyle::NarrowRound,
            border: Some((1.0, DARK_GRAY)),
            bar_color: MID_GRAY,
            handle_color: LIGHT_GRAY,
            highlight: Some(BLUE_HIGHLIGHT),
            width: 30.0,
        }
    }
    pub fn make_vertical(&mut self) -> &mut Self {
        self.orientation = Orientation::Vertical;
        self
    }
    pub fn scrollbar_style(&mut self) -> &mut Self {
        self.variable_handle_size = true;
        self.handle_style = HandleStyle::Square;
        self.bar_style = BarStyle::Wide;
        self.border = None;
        self.bar_color = LIGHT_GRAY;
        self.handle_color = MID_GRAY;
        self.highlight = None;
        self.width = 15.0;
        self
    }
    pub fn set_width(&mut self, width: Scalar) -> &mut Self {
        self.width = width;
        self
    }
    pub fn set_value(&mut self, value: f64) -> &mut Self {
        self.init_value = value;
        self
    }
    pub fn on_value_changed<F>(&mut self, on_value_changed: F) -> &mut Self
        where F: Fn(f64, &mut WidgetEventArgs) + 'static
    {
        self.widget.add_handler_fn(move |event: &MovedSliderWidgetEvent, mut args| {
            let bounds = args.widget.bounds();
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

widget_builder!(SliderBuilder, build: |builder: SliderBuilder| -> WidgetBuilder {

    let (mut slider, mut slider_handle, orientation) = (builder.widget, builder.slider_handle, builder.orientation);

    slider_handle.add_handler(DragHandler::new(orientation, slider.id()));

    match builder.handle_style {
        HandleStyle::Round => {
            slider_handle.set_drawable_with_style(EllipseDrawable::new(), style!(
                EllipseStyleable::BackgroundColor: builder.handle_color,
                EllipseStyleable::Border: builder.border))
        }
        HandleStyle::Square => {
            slider_handle.set_drawable_with_style(RectDrawable::new(), style!(
                RectStyleable::BackgroundColor: builder.handle_color,
                RectStyleable::Border: builder.border))
        }
    };

    let corner_radius = match builder.bar_style {
        BarStyle::NarrowRound => Some(3.0),
        BarStyle::Wide => None,
    };
    let bar_style = style!(
        RectStyleable::BackgroundColor: builder.bar_color,
        RectStyleable::CornerRadius: Some(3.0),
        RectStyleable::Border: builder.border,
        RectStyleable::CornerRadius: corner_radius);

    let pre_style = if let Some(highlight) = builder.highlight {
        style!(parent: bar_style, RectStyleable::BackgroundColor:
            selector!(highlight, INACTIVE: builder.bar_color))
    } else {
        bar_style.clone()
    };
    let mut slider_bar_pre = WidgetBuilder::new();
    slider_bar_pre
        .set_debug_name("slider_bar_pre")
        .set_drawable_with_style(RectDrawable::new(), pre_style);

    let mut slider_bar_post = WidgetBuilder::new();
    slider_bar_post
        .set_debug_name("slider_bar_post")
        .set_drawable_with_style(RectDrawable::new(), bar_style);

    let (bar_width, bar_padding) = match builder.bar_style {
        BarStyle::Wide => (builder.width, 0.0),
        BarStyle::NarrowRound => (builder.width / 3.0, builder.width / 2.0),
    };

    if !builder.variable_handle_size {
        layout!(slider_handle: aspect_ratio(1.0));
    }
    match orientation {
        Orientation::Horizontal => {
            layout!(slider:
                height(builder.width));
            layout!(slider_bar_pre:
                height(bar_width),
                center_vertical(&slider),
                align_left(&slider).padding(bar_padding),
                to_left_of(&slider_handle).padding(-bar_padding));
            layout!(slider_bar_post:
                height(bar_width),
                center_vertical(&slider),
                align_right(&slider).padding(bar_padding),
                to_right_of(&slider_handle).padding(-bar_padding));
            layout!(slider_handle:
                match_height(&slider));

            if builder.variable_handle_size {
                // STRONG + 1.0 for higher strength than handle position
                let mut layout = slider_handle.widget.layout();
                layout.edit_width().set(50.0).strength(STRONG + 1.0);
            }
        }
        Orientation::Vertical => {
            layout!(slider:
                width(builder.width));
            layout!(slider_bar_pre:
                width(bar_width),
                center_horizontal(&slider),
                align_top(&slider).padding(bar_padding),
                above(&slider_handle).padding(-bar_padding));
            layout!(slider_bar_post:
                width(bar_width),
                center_horizontal(&slider),
                align_bottom(&slider).padding(bar_padding),
                below(&slider_handle).padding(-bar_padding));
            layout!(slider_handle:
                match_width(&slider));

            if builder.variable_handle_size {
                // STRONG + 1.0 for higher strength than handle position
                let mut layout = slider_handle.widget.layout();
                layout.edit_height().set(50.0).strength(STRONG + 1.0);
            }
        }
    }
    let handle_id = slider_handle.id();
    slider.add_handler_fn(move |event: &SetSliderValue, args| {
        let bounds = args.widget.bounds();
        let event = SliderHandleInput::SetValue((event.0, bounds));
        event!(Target::Widget(handle_id), event);
    });
    slider.add_handler_fn(move |event: &ClickEvent, args| {
        let bounds = args.widget.bounds();
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

    // need to update position after widget is created because the slider position
    // depends on the measured width of the slider and slider handle
    let init_value = builder.init_value;
    slider.add_handler_fn(move |_: &ChildAttachedEvent, args| {
        event!(Target::WidgetRef(args.widget), SetSliderValue(init_value));
    });
    slider
});

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
    fn handle(&mut self, event: &SliderHandleInput, mut args: WidgetEventArgs) {
        let bounds = args.widget.bounds();
        let handle_radius = if let Orientation::Horizontal = self.orientation {
            bounds.width() / 2.0
        } else {
            bounds.height() / 2.0
        };
        match *event {
            SliderHandleInput::WidgetDrag(ref event) => {
                if args.widget.props().contains(&Property::Inactive) {
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
                        });
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
                if value.is_finite() {
                    if let Orientation::Horizontal = self.orientation {
                        let pos = parent_bounds.left() + value * (parent_bounds.width() - bounds.width());
                        args.widget.update_layout(|layout| {
                            layout.edit_left().set(pos);
                        });
                    } else {
                        let pos = parent_bounds.top() + value * (parent_bounds.height() - bounds.height());
                        args.widget.update_layout(|layout| {
                            layout.edit_top().set(pos);
                        });
                    }
                }
            }
            SliderHandleInput::SliderClicked((position, parent_bounds)) => {
                if args.widget.props().contains(&Property::Inactive) {
                    return;
                }
                let position = if let Orientation::Horizontal = self.orientation {
                    let min = parent_bounds.left() + handle_radius;
                    let max = parent_bounds.left() + parent_bounds.width() - handle_radius;
                    let position = f64::min(f64::max(position, min), max);
                    args.widget.update_layout(|layout| {
                        layout.edit_left().set(position - handle_radius);
                    });
                    position
                } else {
                    let min = parent_bounds.top() + handle_radius;
                    let max = parent_bounds.top() + parent_bounds.height() - handle_radius;
                    let position = f64::min(f64::max(position, min), max);
                    args.widget.update_layout(|layout| {
                        layout.edit_top().set(position - handle_radius);
                    });
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
