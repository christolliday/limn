use cassowary::strength::*;

use layout::constraint::*;
use input::mouse::ClickEvent;
use event::{WidgetEventHandler, WidgetEventArgs};
use widget::Widget;
use widget::property::Property;
use widget::property::states::*;
use widgets::drag::{DragEvent, WidgetDrag};
use drawable::rect::{RectDrawable, RectStyleable};
use drawable::ellipse::{EllipseDrawable, EllipseStyleable};
use ui::ChildAttachedEvent;
use util::{Rect, RectExt};
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
    pub widget: Widget,
    pub slider_handle: Widget,
    pub orientation: Orientation,
    pub init_value: f32,
    pub variable_handle_size: bool,
    pub handle_style: HandleStyle,
    pub bar_style: BarStyle,
    pub border: Option<(f32, Color)>,
    pub bar_color: Color,
    pub handle_color: Color,
    pub highlight: Option<Color>,
    pub width: f32,
}

impl SliderBuilder {
    pub fn new() -> Self {
        let mut slider = Widget::new();
        slider.set_debug_name("slider");

        let mut slider_handle = Widget::new();
        slider_handle
            .set_debug_name("slider_handle")
            .add_handler_fn(|event: &WidgetDrag, args| {
                args.widget.event(SliderHandleInput::WidgetDrag(event.clone()));
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
            border: Some((1.0, GRAY_30)),
            bar_color: GRAY_70,
            handle_color: GRAY_80,
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
        self.bar_color = GRAY_80;
        self.handle_color = GRAY_70;
        self.highlight = None;
        self.width = 15.0;
        self
    }
    pub fn set_width(&mut self, width: f32) -> &mut Self {
        self.width = width;
        self
    }
    pub fn set_value(&mut self, value: f32) -> &mut Self {
        self.init_value = value;
        self
    }
    pub fn on_value_changed<F>(&mut self, on_value_changed: F) -> &mut Self
        where F: Fn(f32, &mut WidgetEventArgs) + 'static
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
    pub fn build(self) -> Widget {
        let (mut slider, mut slider_handle, orientation) = (self.widget, self.slider_handle, self.orientation);

        slider_handle.add_handler(DragHandler::new(orientation, slider.clone()));

        match self.handle_style {
            HandleStyle::Round => {
                slider_handle.set_drawable_with_style(EllipseDrawable::new(), style!(
                    EllipseStyleable::BackgroundColor: self.handle_color,
                    EllipseStyleable::Border: self.border))
            }
            HandleStyle::Square => {
                slider_handle.set_drawable_with_style(RectDrawable::new(), style!(
                    RectStyleable::BackgroundColor: self.handle_color,
                    RectStyleable::Border: self.border))
            }
        };

        let corner_radius = match self.bar_style {
            BarStyle::NarrowRound => Some(3.0),
            BarStyle::Wide => None,
        };
        let bar_style = style!(
            RectStyleable::BackgroundColor: self.bar_color,
            RectStyleable::CornerRadius: Some(3.0),
            RectStyleable::Border: self.border,
            RectStyleable::CornerRadius: corner_radius);

        let pre_style = if let Some(highlight) = self.highlight {
            style!(parent: bar_style, RectStyleable::BackgroundColor:
                selector!(highlight, INACTIVE: self.bar_color))
        } else {
            bar_style.clone()
        };
        let mut slider_bar_pre = Widget::new();
        slider_bar_pre
            .set_debug_name("slider_bar_pre")
            .set_drawable_with_style(RectDrawable::new(), pre_style);

        let mut slider_bar_post = Widget::new();
        slider_bar_post
            .set_debug_name("slider_bar_post")
            .set_drawable_with_style(RectDrawable::new(), bar_style);

        let (bar_width, bar_padding) = match self.bar_style {
            BarStyle::Wide => (self.width, 0.0),
            BarStyle::NarrowRound => (self.width / 3.0, self.width / 2.0),
        };

        if !self.variable_handle_size {
            slider_handle.layout().add(aspect_ratio(1.0));
        }
        match orientation {
            Orientation::Horizontal => {
                slider.layout().add(height(self.width));
                slider_bar_pre.layout().add(constraints![
                    height(bar_width),
                    center_vertical(&slider),
                    align_left(&slider).padding(bar_padding),
                    to_left_of(&slider_handle).padding(-bar_padding),
                ]);
                slider_bar_post.layout().add(constraints![
                    height(bar_width),
                    center_vertical(&slider),
                    align_right(&slider).padding(bar_padding),
                    to_right_of(&slider_handle).padding(-bar_padding),
                ]);
                slider_handle.layout().add(match_height(&slider));

                if self.variable_handle_size {
                    // STRONG + 1.0 for higher strength than handle position
                    let mut layout = slider_handle.layout();
                    layout.edit_width().set(50.0).strength(STRONG + 1.0);
                }
            }
            Orientation::Vertical => {
                slider.layout().add(width(self.width));
                slider_bar_pre.layout().add(constraints![
                    width(bar_width),
                    center_horizontal(&slider),
                    align_top(&slider).padding(bar_padding),
                    above(&slider_handle).padding(-bar_padding),
                ]);
                slider_bar_post.layout().add(constraints![
                    width(bar_width),
                    center_horizontal(&slider),
                    align_bottom(&slider).padding(bar_padding),
                    below(&slider_handle).padding(-bar_padding),
                ]);

                slider_handle.layout().add(match_width(&slider));

                if self.variable_handle_size {
                    // STRONG + 1.0 for higher strength than handle position
                    let mut layout = slider_handle.layout();
                    layout.edit_height().set(50.0).strength(STRONG + 1.0);
                }
            }
        }
        let handle_ref = slider_handle.clone();
        slider.add_handler_fn(move |event: &SetSliderValue, args| {
            let bounds = args.widget.bounds();
            let event = SliderHandleInput::SetValue((event.0, bounds));
            handle_ref.event(event);
        });
        let handle_ref = slider_handle.clone();
        slider.add_handler_fn(move |event: &ClickEvent, args| {
            let bounds = args.widget.bounds();
            let position = if let Orientation::Horizontal = orientation {
                event.position.x
            } else {
                event.position.y
            };
            let event = SliderHandleInput::SliderClicked((position, bounds));
            handle_ref.event(event);
        });
        slider.add_child(slider_bar_pre);
        slider.add_child(slider_bar_post);
        slider.add_child(slider_handle);

        // need to update position after widget is created because the slider position
        // depends on the measured width of the slider and slider handle
        let init_value = self.init_value;
        slider.add_handler_fn(move |_: &ChildAttachedEvent, args| {
            args.widget.event(SetSliderValue(init_value));
        });
        slider
    }
}

widget_builder!(SliderBuilder);

struct MovedSliderWidgetEvent {
    orientation: Orientation,
    slider_pos: f32,
    handle_size: f32,
}

pub struct SetSliderValue(pub f32);
pub enum SliderHandleInput {
    WidgetDrag(WidgetDrag),
    SetValue((f32, Rect)),
    SliderClicked((f32, Rect)),
}
struct DragHandler {
    orientation: Orientation,
    container: Widget,
    start_pos: f32,
}
impl DragHandler {
    pub fn new(orientation: Orientation, container: Widget) -> Self {
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
                        self.container.event(event);
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
                    let position = f32::min(f32::max(position, min), max);
                    args.widget.update_layout(|layout| {
                        layout.edit_left().set(position - handle_radius);
                    });
                    position
                } else {
                    let min = parent_bounds.top() + handle_radius;
                    let max = parent_bounds.top() + parent_bounds.height() - handle_radius;
                    let position = f32::min(f32::max(position, min), max);
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
                self.container.event(event);
            }
        }
    }
}
