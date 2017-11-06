use std::ops::Range;

use cassowary::strength::*;

use layout::constraint::*;
use layout::LayoutUpdated;
use input::mouse::ClickEvent;
use input::drag::{DragEvent, DragState};
use event::{EventHandler, EventArgs};
use widget::{WidgetBuilder, WidgetRef};
use widget::property::Property;
use widget::property::states::*;
use widget::style::Value;
use draw::rect::RectComponentStyle;
use draw::ellipse::EllipseComponentStyle;
use geometry::{RectExt, Point};
use color::*;
use style::ComponentStyle;

#[derive(Debug, Clone, Copy)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy)]
pub enum HandleStyle {
    Round,
    Square
}

#[derive(Debug, Clone, Copy)]
pub enum BarStyle {
    NarrowRound,
    Wide,
}

#[derive(Debug)]
pub struct SliderBuilder {
    pub widget: WidgetBuilder,
    pub slider_handle: WidgetBuilder,
    pub orientation: Orientation,
    pub range: Range<f32>,
    pub init_value: Option<f32>,
    pub variable_handle_size: bool,
    pub handle_style: HandleStyle,
    pub bar_style: BarStyle,
    pub border: Option<(f32, Color)>,
    pub bar_color: Color,
    pub handle_color: Color,
    pub highlight: Option<Color>,
    pub width: f32,
}

impl Default for SliderBuilder {
    #[inline]
    fn default() -> Self {
        let widget = WidgetBuilder::new("slider");

        let slider_handle = WidgetBuilder::new("slider_handle");
        SliderBuilder {
            widget: widget,
            slider_handle: slider_handle,
            orientation: Orientation::Horizontal,
            range: 0.0..1.0,
            init_value: None,
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
}

impl SliderBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the orientation of the slider to vertical
    pub fn make_vertical(&mut self) -> &mut Self {
        self.orientation = Orientation::Vertical;
        self
    }

    /// Sets the scrollbar style
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
        self.init_value = Some(value);
        self
    }
    pub fn set_range(&mut self, range: Range<f32>) -> &mut Self {
        self.range = range;
        self
    }
    pub fn on_value_changed<F>(&mut self, on_value_changed: F) -> &mut Self
        where F: Fn(f32, &mut EventArgs) + 'static
    {
        self.add_handler(move |event: &SliderEvent, mut args: EventArgs| {
            on_value_changed(event.value, &mut args);
        });
        self
    }
}

widget_builder!(SliderBuilder);
impl Into<WidgetBuilder> for SliderBuilder {
    fn into(self) -> WidgetBuilder {
        let (mut widget, mut slider_handle, orientation) = (self.widget, self.slider_handle, self.orientation);

        match self.handle_style {
            HandleStyle::Round => {
                slider_handle.set_draw_style(EllipseComponentStyle {
                    background_color: Some(Value::from(self.handle_color)),
                    border: Some(Value::from(self.border)),
                    ..EllipseComponentStyle::default()
                });
            }
            HandleStyle::Square => {
                slider_handle.set_draw_style(RectComponentStyle {
                    background_color: Some(Value::from(self.handle_color)),
                    border: Some(Value::from(self.border)),
                    ..RectComponentStyle::default()
                });
            }
        };

        let corner_radius = match self.bar_style {
            BarStyle::NarrowRound => Some(3.0),
            BarStyle::Wide => None,
        };
        let bar_style = RectComponentStyle {
            background_color: Some(Value::from(self.handle_color)),
            corner_radius: Some(Value::from(Some(3.0))),
            border: Some(Value::from(self.border)),
            ..RectComponentStyle::default()
        };

        let pre_style = if let Some(highlight) = self.highlight {
            RectComponentStyle {
                background_color: Some(Value::from(selector!(highlight, INACTIVE: self.bar_color))),
                ..RectComponentStyle::default()
            }.merge(&bar_style)
            //style!(parent: bar_style, RectStyle::BackgroundColor:
            //    selector!(highlight, INACTIVE: self.bar_color))
        } else {
            bar_style.clone()
        };
        let mut slider_bar_pre = WidgetBuilder::new("slider_bar_pre");
        slider_bar_pre.set_draw_style(pre_style);

        let mut slider_bar_post = WidgetBuilder::new("slider_bar_post");
        slider_bar_post.set_draw_style(bar_style);

        let (bar_width, bar_padding) = match self.bar_style {
            BarStyle::Wide => (self.width, 0.0),
            BarStyle::NarrowRound => (self.width / 3.0, self.width / 2.0),
        };

        if !self.variable_handle_size {
            slider_handle.layout().add(aspect_ratio(1.0));
        }
        match orientation {
            Orientation::Horizontal => {
                widget.layout().add(height(self.width));
                slider_bar_pre.layout().add(constraints![
                    height(bar_width),
                    center_vertical(&widget),
                    align_left(&widget).padding(bar_padding),
                    to_left_of(&slider_handle).padding(-bar_padding),
                ]);
                slider_bar_post.layout().add(constraints![
                    height(bar_width),
                    center_vertical(&widget),
                    align_right(&widget).padding(bar_padding),
                    to_right_of(&slider_handle).padding(-bar_padding),
                ]);
                slider_handle.layout().add(match_height(&widget));

                if self.variable_handle_size {
                    // STRONG + 1.0 for higher strength than handle position
                    let mut layout = slider_handle.layout();
                    layout.edit_width().set(50.0).strength(STRONG + 1.0);
                }
            }
            Orientation::Vertical => {
                widget.layout().add(width(self.width));
                slider_bar_pre.layout().add(constraints![
                    width(bar_width),
                    center_horizontal(&widget),
                    align_top(&widget).padding(bar_padding),
                    above(&slider_handle).padding(-bar_padding),
                ]);
                slider_bar_post.layout().add(constraints![
                    width(bar_width),
                    center_horizontal(&widget),
                    align_bottom(&widget).padding(bar_padding),
                    below(&slider_handle).padding(-bar_padding),
                ]);

                slider_handle.layout().add(match_width(&widget));

                if self.variable_handle_size {
                    // STRONG + 1.0 for higher strength than handle position
                    let mut layout = slider_handle.layout();
                    layout.edit_height().set(50.0).strength(STRONG + 1.0);
                }
            }
        }

        let widget_ref = widget.widget_ref();
        slider_handle
            .add_handler(move |event: &DragEvent, _: EventArgs| {
                widget_ref.event(SliderInputEvent::Drag(*event));
            })
            .make_draggable();

        let widget_ref = widget.widget_ref();
        slider_bar_pre.add_handler(move |event: &ClickEvent, _: EventArgs| {
            widget_ref.event(SliderInputEvent::Click(event.position));
        });
        let widget_ref = widget.widget_ref();
        slider_bar_post.add_handler(move |event: &ClickEvent, _: EventArgs| {
            widget_ref.event(SliderInputEvent::Click(event.position));
        });
        widget.add_handler(move |event: &SetSliderValue, args: EventArgs| {
            args.widget.event(SliderInputEvent::SetValue(event.0));
        });
        widget.add_handler(move |event: &SetSliderRange, args: EventArgs| {
            args.widget.event(SliderInputEvent::SetRange(event.0.clone()));
        });
        widget.add_handler(move |_: &LayoutUpdated, args: EventArgs| {
            args.widget.event(SliderInputEvent::LayoutUpdated);
        });
        let widget_ref = widget.widget_ref();
        widget.add_handler(SliderHandler::new(orientation, self.range, widget_ref.clone(), slider_handle.widget_ref(), self.init_value));

        widget.add_child(slider_bar_pre);
        widget.add_child(slider_bar_post);
        widget.add_child(slider_handle);
        widget
    }
}

#[derive(Debug, Copy, Clone)]
pub struct SliderEvent {
    pub value: f32,
    pub offset: f32,
    pub dragging: bool,
}

#[derive(Debug, Copy, Clone)]
pub struct SetSliderValue(pub f32);
pub struct SetSliderRange(pub Range<f32>);

#[derive(Debug, Clone)]
enum SliderInputEvent {
    Drag(DragEvent),
    Click(Point),
    SetValue(f32),
    SetRange(Range<f32>),
    LayoutUpdated,
}

#[derive(Debug, Clone)]
struct SliderHandler {
    orientation: Orientation,
    range: Range<f32>,
    slider_ref: WidgetRef,
    handle_ref: WidgetRef,
    drag_start_pos: f32,
    drag_start_val: f32,
    last_val: f32,
}

impl SliderHandler {
    fn new(orientation: Orientation, range: Range<f32>, slider_ref: WidgetRef, handle_ref: WidgetRef, init_value: Option<f32>) -> Self {
        let value = init_value.unwrap_or(range.start);
        let handler = SliderHandler {
            orientation: orientation,
            range: range,
            slider_ref: slider_ref,
            handle_ref: handle_ref,
            drag_start_pos: 0.0,
            drag_start_val: 0.0,
            last_val: value,
        };
        handler.update_handle_pos(value);
        handler
    }
    fn get_value_for_pos(&self, handle_pos: f32) -> f32 {
        let handle_size = self.handle_size();
        let handle_pos_range = self.slider_size() - handle_size;
        let slider_range = self.slider_range();
        let val = (handle_pos - handle_size / 2.0 - slider_range.start) / handle_pos_range;
        val * (self.range.end - self.range.start) + self.range.start
    }
    fn slider_size(&self) -> f32 {
        if let Orientation::Horizontal = self.orientation {
            self.slider_ref.bounds().width()
        } else {
            self.slider_ref.bounds().height()
        }
    }
    fn slider_range(&self) -> Range<f32> {
        let bounds = self.slider_ref.bounds();
        if let Orientation::Horizontal = self.orientation {
            (bounds.left()..bounds.right())
        } else {
            (bounds.top()..bounds.bottom())
        }
    }
    fn handle_size(&self) -> f32 {
        if let Orientation::Horizontal = self.orientation {
            self.handle_ref.bounds().width()
        } else {
            self.handle_ref.bounds().height()
        }
    }
    fn handle_range(&self) -> Range<f32> {
        let bounds = self.handle_ref.bounds();
        if let Orientation::Horizontal = self.orientation {
            (bounds.left()..bounds.right())
        } else {
            (bounds.top()..bounds.bottom())
        }
    }
    fn clamp_position(&self, handle_pos: f32) -> f32 {
        let handle_size = self.handle_size();
        let slider_range = self.slider_range();
        let min = slider_range.start + handle_size / 2.0;
        let max = slider_range.end - handle_size / 2.0;
        f32::min(f32::max(handle_pos, min), max)
    }
    fn update_handle_pos(&self, value: f32) {
        let value = (value - self.range.start) / (self.range.end - self.range.start);
        let range_of_motion = self.slider_size() - self.handle_size();
        let handle_start = self.slider_range().start + value * range_of_motion;
        self.handle_ref.update_layout(|layout| {
            if let Orientation::Horizontal = self.orientation {
                layout.edit_left().set(handle_start).strength(WEAK);
            } else {
                layout.edit_top().set(handle_start).strength(WEAK);
            }
        });
    }
}

impl EventHandler<SliderInputEvent> for SliderHandler {
    fn handle(&mut self, event: &SliderInputEvent, args: EventArgs) {
        match *event {
            SliderInputEvent::Drag(ref event) => {
                if args.widget.props().contains(&Property::Inactive) {
                    return;
                }
                let &DragEvent { ref state, offset, .. } = event;
                let offset = if let Orientation::Horizontal = self.orientation {
                    offset.x
                } else {
                    offset.y
                };
                if *state == DragState::Start {
                    self.drag_start_pos = self.handle_range().start;
                    self.drag_start_val = self.get_value_for_pos(self.handle_range().start + self.handle_size() / 2.0);
                } else {
                    let handle_start = self.drag_start_pos + offset;
                    let position = self.clamp_position(handle_start + self.handle_size() / 2.0);
                    let value = self.get_value_for_pos(position);
                    self.update_handle_pos(value);
                    let dragging = *state != DragState::End;
                    let event = SliderEvent {
                        value: value,
                        offset: value - self.drag_start_val,
                        dragging: dragging,
                    };
                    self.slider_ref.event(event);
                    if !dragging {
                        self.last_val = value;
                    }
                }
            }
            SliderInputEvent::Click(point) => {
                if args.widget.props().contains(&Property::Inactive) {
                    return;
                }
                let position = if let Orientation::Horizontal = self.orientation {
                    point.x
                } else {
                    point.y
                };
                let position = self.clamp_position(position);
                let value = self.get_value_for_pos(position);
                self.update_handle_pos(value);
                let event = SliderEvent {
                    value: value,
                    offset: value - self.last_val,
                    dragging: false,
                };
                self.slider_ref.event(event);
                self.last_val = value;
            }
            SliderInputEvent::SetValue(value) => {
                if value.is_finite() {
                    self.last_val = value;
                    self.update_handle_pos(value);
                }
            },
            SliderInputEvent::SetRange(ref range) => {
                self.range = range.clone();
                self.last_val = range.start;
                self.update_handle_pos(range.start);
            },
            SliderInputEvent::LayoutUpdated => {
                self.update_handle_pos(self.last_val);
            }
        }
    }
}
