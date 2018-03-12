use std::ops::Range;

use prelude::*;
use draw::prelude::*;

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

component_style!{pub struct Slider<name="slider", style=SliderStyle> {
    orientation: Orientation = Orientation::Horizontal,
    range: Range<f32> = 0.0..1.0,
    init_value: Option<f32> = None,
    variable_handle_size: bool = false,
    handle_style: HandleStyle = HandleStyle::Round,
    bar_style: BarStyle = BarStyle::NarrowRound,
    border: Option<(f32, Color)> = Some((1.0, GRAY_30)),
    bar_color: Color = GRAY_70,
    handle_color: Color = GRAY_80,
    highlight: Option<Color> = Some(BLUE_HIGHLIGHT),
    width: f32 = 30.0,
}}

impl Slider {
    /// Sets the orientation of the slider to vertical
    pub fn make_vertical(&mut self) -> &mut Self {
        self.orientation = Orientation::Vertical;
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
}

impl WidgetModifier for Slider {
    fn apply(&self, widget: &mut Widget) {

        let mut slider_handle = Widget::new("slider_handle");
        match self.handle_style {
            HandleStyle::Round => {
                slider_handle.set_draw_style(style!(EllipseStyle {
                    background_color: self.handle_color,
                    border: self.border,
                }));
            }
            HandleStyle::Square => {
                slider_handle.set_draw_style(style!(RectStyle {
                    background_color: self.handle_color,
                    border: self.border,
                }));
            }
        };

        let corner_radius = match self.bar_style {
            BarStyle::NarrowRound => Some(3.0),
            BarStyle::Wide => None,
        };
        let bar_style = style!(RectStyle {
            background_color: self.bar_color,
            corner_radius: corner_radius,
            border: self.border,
        });

        let mut slider_bar_pre = Widget::new("slider_bar_pre");
        if let Some(highlight) = self.highlight {
            let mut style = DrawStyle::from(RectStyle {
                background_color: Some(highlight),
                ..bar_style
            });
            style.prop_style(INACTIVE.clone(), style!(RectStyle {
                background_color: self.bar_color,
            }));
            slider_bar_pre.set_draw_style(style);
        } else {
            slider_bar_pre.set_draw_style(bar_style.clone());
        }

        let mut slider_bar_post = Widget::new("slider_bar_post");
        slider_bar_post.set_draw_style(bar_style);

        let (bar_width, bar_padding) = match self.bar_style {
            BarStyle::Wide => (self.width, 0.0),
            BarStyle::NarrowRound => (self.width / 3.0, self.width / 2.0),
        };

        if !self.variable_handle_size {
            slider_handle.layout().add(aspect_ratio(1.0));
        }
        match self.orientation {
            Orientation::Horizontal => {
                widget.layout().add(height(self.width));
                slider_bar_pre.layout().add(constraints![
                    height(bar_width),
                    center_vertical(widget),
                    align_left(widget).padding(bar_padding),
                    to_left_of(&slider_handle).padding(-bar_padding),
                ]);
                slider_bar_post.layout().add(constraints![
                    height(bar_width),
                    center_vertical(widget),
                    align_right(widget).padding(bar_padding),
                    to_right_of(&slider_handle).padding(-bar_padding),
                ]);
                slider_handle.layout().add(match_height(widget));

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
                    center_horizontal(widget),
                    align_top(widget).padding(bar_padding),
                    above(&slider_handle).padding(-bar_padding),
                ]);
                slider_bar_post.layout().add(constraints![
                    width(bar_width),
                    center_horizontal(widget),
                    align_bottom(widget).padding(bar_padding),
                    below(&slider_handle).padding(-bar_padding),
                ]);

                slider_handle.layout().add(match_width(widget));

                if self.variable_handle_size {
                    // STRONG + 1.0 for higher strength than handle position
                    let mut layout = slider_handle.layout();
                    layout.edit_height().set(50.0).strength(STRONG + 1.0);
                }
            }
        }
        slider_handle.make_draggable();

        forward_event!(DragEvent: slider_handle -> SliderInputEvent: widget);
        forward_event!(ClickEvent: slider_bar_pre -> SliderInputEvent: widget);
        forward_event!(ClickEvent: slider_bar_post -> SliderInputEvent: widget);
        forward_event!(SetSliderValue: widget -> SliderInputEvent: widget);
        forward_event!(SetSliderRange: widget -> SliderInputEvent: widget);
        forward_event!(LayoutUpdated: widget -> SliderInputEvent: widget);
        let widget_c = widget.clone();
        let handle_c = slider_handle.clone();
        widget.add_handler(SliderHandler::new(self.orientation, self.range.clone(), widget_c, handle_c, self.init_value));

        widget.add_child(slider_bar_pre);
        widget.add_child(slider_bar_post);
        widget.add_child(slider_handle);
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
#[derive(Debug, Clone)]
pub struct SetSliderRange(pub Range<f32>);

multi_event!{impl EventHandler<SliderInputEvent> for SliderHandler {
    DragEvent => drag,
    ClickEvent => click_bar,
    SetSliderValue => set_value,
    SetSliderRange => set_range,
    LayoutUpdated => layout_updated,
}}

#[derive(Debug, Clone)]
struct SliderHandler {
    orientation: Orientation,
    range: Range<f32>,
    slider_ref: Widget,
    handle_ref: Widget,
    drag_start_pos: f32,
    drag_start_val: f32,
    last_val: f32,
}

impl SliderHandler {
    fn new(orientation: Orientation, range: Range<f32>, slider_ref: Widget, handle_ref: Widget, init_value: Option<f32>) -> Self {
        let value = init_value.unwrap_or(range.start);
        let mut handler = SliderHandler {
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
    fn update_handle_pos(&mut self, value: f32) {
        let value = (value - self.range.start) / (self.range.end - self.range.start);
        let range_of_motion = self.slider_size() - self.handle_size();
        let handle_start = self.slider_range().start + value * range_of_motion;
        let mut layout = self.handle_ref.layout();
        if let Orientation::Horizontal = self.orientation {
            layout.edit_left().set(handle_start).strength(WEAK);
        } else {
            layout.edit_top().set(handle_start).strength(WEAK);
        }
    }

    fn drag(&mut self, event: &DragEvent, args: EventArgs) {
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

    fn click_bar(&mut self, event: &ClickEvent, args: EventArgs) {
        if args.widget.props().contains(&Property::Inactive) {
            return;
        }
        let position = if let Orientation::Horizontal = self.orientation {
            event.position.x
        } else {
            event.position.y
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

    fn set_value(&mut self, event: &SetSliderValue, _: EventArgs) {
        let SetSliderValue(value) = *event;
        if value.is_finite() {
            self.last_val = value;
            self.update_handle_pos(value);
        }
    }
    fn set_range(&mut self, event: &SetSliderRange, _: EventArgs) {
        let &SetSliderRange(ref range) = event;
        self.range = range.clone();
        self.last_val = range.start;
        self.update_handle_pos(range.start);
    }
    fn layout_updated(&mut self, _: &LayoutUpdated, _: EventArgs) {
        let last_val = self.last_val;
        self.update_handle_pos(last_val);
    }
}
