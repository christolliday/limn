use glutin;
use cassowary::strength::*;
use cassowary::WeightedRelation::*;

use layout::constraint::*;
use event::{EventArgs, EventHandler};
use widget::{WidgetBuilder, WidgetRef};
use widgets::slider::{Slider, SliderEvent, SetSliderValue};
use geometry::{Size, Vector, Rect, RectExt};
use layout::{LayoutUpdated, LAYOUT};
use input::mouse::WidgetMouseWheel;
use draw::rect::RectStyle;
use color::*;
use style::WidgetModifier;

component_style!{pub struct ScrollContainer<name="scroll", style=ScrollStyle> {
    content: Option<WidgetRef> = None,
    has_scrollbars: bool = false,
}}

impl ScrollContainer {
    /// Set the scrollable content
    pub fn add_content(&mut self, widget: WidgetRef) -> &mut Self {
        self.content = Some(widget);
        self
    }
    /// Display vertical and horizontal scrollbars
    pub fn add_scrollbar(&mut self) -> &mut Self {
        self.has_scrollbars = true;
        self
    }
}

impl WidgetModifier for ScrollContainer {
    fn apply(&self, widget: &mut WidgetBuilder) {
        let mut content_holder = WidgetBuilder::new("content_holder");
        content_holder.layout().no_container();
        let widget_ref = content_holder.widget_ref();
        content_holder.add_handler(move |_: &LayoutUpdated, _: EventArgs| {
            widget_ref.event(ScrollParentEvent::ContainerLayoutUpdated);
        });
        let mut content = self.content.clone().expect("Scroll bar has no content");
        let widget_ref = content_holder.widget_ref();
        content.add_handler(move |_: &LayoutUpdated, args: EventArgs| {
            widget_ref.event(ScrollParentEvent::ContentLayoutUpdated(args.widget.bounds()));
        });
        content_holder.layout().add(constraints![
            match_layout(widget).strength(STRONG)
        ]);
        {
            let content_holder = &content_holder.layout().vars;
            content.update_layout(|layout| {
                layout.add(constraints![
                    LAYOUT.left | LE(REQUIRED) | content_holder.left,
                    LAYOUT.top | LE(REQUIRED) | content_holder.top,
                    LAYOUT.left | EQ(WEAK) | content_holder.left,
                    LAYOUT.top | EQ(WEAK) | content_holder.top,
                    LAYOUT.right | GE(STRONG) | content_holder.right,
                    LAYOUT.bottom | GE(STRONG) | content_holder.bottom,
                ]);
            });
        }
        let mut scrollbars = if self.has_scrollbars {
            let mut scrollbar_h = Slider::default();
            scrollbar_h.scrollbar_style();
            let mut scrollbar_h = WidgetBuilder::from_modifier(scrollbar_h);
            scrollbar_h.set_name("scrollbar_h");
            scrollbar_h.layout().add(constraints![
                align_bottom(widget),
                align_left(widget),
                align_below(&content_holder),
            ]);
            let mut scrollbar_v = Slider::default();
            scrollbar_v.make_vertical().scrollbar_style();
            let mut scrollbar_v = WidgetBuilder::from_modifier(scrollbar_v);
            scrollbar_v.set_name("scrollbar_v");
            scrollbar_v.layout().add(constraints![
                align_right(widget),
                align_top(widget),
                align_to_right_of(&content_holder),
            ]);
            let corner_style = style!(RectStyle {
                background_color: GRAY_70,
            });
            let mut corner = WidgetBuilder::new("corner");
            corner.set_draw_style(corner_style);
            corner.layout().add(constraints![
                align_bottom(widget),
                align_right(widget),
                align_to_right_of(&scrollbar_h),
                align_below(&scrollbar_v),
                match_height(&scrollbar_h),
                match_width(&scrollbar_v),
            ]);
            let widget_ref = content_holder.widget_ref();
            scrollbar_h.add_handler(move |event: &SliderEvent, _: EventArgs| {
                widget_ref.event(ScrollParentEvent::ScrollBarMovedX(event.value));
            });
            let widget_ref = content_holder.widget_ref();
            scrollbar_v.add_handler(move |event: &SliderEvent, _: EventArgs| {
                widget_ref.event(ScrollParentEvent::ScrollBarMovedY(event.value));
            });

            Some((corner, scrollbar_h, scrollbar_v))
        } else {
            None
        };

        let mut scroll_parent_handler = ScrollParent::new(&mut content);
        if let Some((ref mut corner, ref mut scrollbar_h, ref mut scrollbar_v)) = scrollbars {
            scroll_parent_handler.scrollbars = Some(ScrollBars::new(scrollbar_h, scrollbar_v, corner.widget_ref()));
        }
        content_holder.add_handler(scroll_parent_handler);
        content_holder.add_handler(|event: &WidgetMouseWheel, args: EventArgs| {
            args.widget.event(ScrollParentEvent::WidgetMouseWheel(*event));
        });
        content_holder.add_child(content);
        if scrollbars.is_some() {
            content_holder.layout().add(constraints![
                align_left(widget),
                align_top(widget),
            ]);
        } else {
            content_holder.layout().add(constraints![
                match_layout(widget),
            ]);
        }
        widget.add_child(content_holder);
        if let Some((corner, scrollbar_h, scrollbar_v)) = scrollbars {
            widget.add_child(corner);
            widget.add_child(scrollbar_h);
            widget.add_child(scrollbar_v);
        }
    }
}

#[allow(dead_code)]
struct ScrollBars {
    scrollbar_h: WidgetRef,
    scrollbar_v: WidgetRef,
    corner: WidgetRef,
    h_handle: WidgetRef,
    v_handle: WidgetRef,
}
impl ScrollBars {
    fn new(scrollbar_h: &mut WidgetBuilder, scrollbar_v: &mut WidgetBuilder, corner: WidgetRef) -> Self {
        ScrollBars {
            scrollbar_h: scrollbar_h.widget_ref(),
            scrollbar_v: scrollbar_v.widget_ref(),
            corner: corner,
            h_handle: scrollbar_h.widget_ref().child("slider_handle").unwrap(),
            v_handle: scrollbar_v.widget_ref().child("slider_handle").unwrap(),
        }
    }
}

enum ScrollParentEvent {
    ContainerLayoutUpdated,
    ContentLayoutUpdated(Rect),
    WidgetMouseWheel(WidgetMouseWheel),
    ScrollBarMovedX(f32),
    ScrollBarMovedY(f32),
}
struct ScrollParent {
    scrollable: WidgetRef,
    content_rect: Rect,
    container_rect: Rect,
    width_ratio: f32,
    height_ratio: f32,
    scrollable_area: Size,
    offset: Vector,
    pub scrollbars: Option<ScrollBars>,
}
impl ScrollParent {
    fn new(scrollable: &mut WidgetRef) -> Self {
        ScrollParent {
            scrollable: scrollable.clone(),
            content_rect: Rect::zero(),
            container_rect: Rect::zero(),
            width_ratio: 0.0,
            height_ratio: 0.0,
            scrollable_area: Size::zero(),
            offset: Vector::zero(),
            scrollbars: None,
        }
    }
    fn move_content_x(&mut self) {
        let scroll_to = self.container_rect.left() + self.offset.x;
        self.scrollable.update_layout(|layout| {
            layout.edit_left().set(scroll_to);
        });
    }
    fn move_content_y(&mut self) {
        let scroll_to = self.container_rect.top() + self.offset.y;
        self.scrollable.update_layout(|layout| {
            layout.edit_top().set(scroll_to);
        });
    }
    fn move_slider_x(&mut self) {
        if let Some(ref mut scrollbars) = self.scrollbars {
            let offset_x = -self.offset.x / self.scrollable_area.width;
            scrollbars.scrollbar_h.event(SetSliderValue(offset_x));
        }
    }
    fn move_slider_y(&mut self) {
        if let Some(ref mut scrollbars) = self.scrollbars {
            let offset_y = -self.offset.y / self.scrollable_area.height;
            scrollbars.scrollbar_v.event(SetSliderValue(offset_y));
        }
    }
}
impl EventHandler<ScrollParentEvent> for ScrollParent {
    fn handle(&mut self, event: &ScrollParentEvent, args: EventArgs) {
        match *event {
            ScrollParentEvent::ContainerLayoutUpdated | ScrollParentEvent::ContentLayoutUpdated(_) => {

                if let ScrollParentEvent::ContentLayoutUpdated(rect) = *event {
                    self.content_rect = rect
                }
                if let ScrollParentEvent::ContainerLayoutUpdated = *event {
                    self.container_rect = args.widget.bounds();
                }

                let scrollable_area = self.content_rect.size - self.container_rect.size;
                let content_offset = self.content_rect.origin - self.container_rect.origin;
                if content_offset != self.offset || scrollable_area != self.scrollable_area {
                    self.offset = content_offset;
                    self.scrollable_area = scrollable_area;
                    if self.scrollable_area.width > 0.0 {
                        self.move_slider_x();
                    }
                    if self.scrollable_area.height > 0.0 {
                        self.move_slider_y();
                    }
                }


                let width_ratio = self.container_rect.width() / self.content_rect.width();
                let height_ratio = self.container_rect.height() / self.content_rect.height();
                if let Some(ref mut scrollbars) = self.scrollbars {

                    // update handle sizes
                    let mut visibility_updated = false;

                    if width_ratio.is_finite() && (width_ratio - self.width_ratio).abs() > ::std::f32::EPSILON {
                        let width = self.container_rect.width() * width_ratio;
                        scrollbars.h_handle.update_layout(|layout| {
                            layout.edit_width().set(width);
                        });
                        let scrollbar_hidden = scrollbars.scrollbar_h.layout().hidden;
                        let hide_scrollbar = width_ratio >= 1.0 && !scrollbar_hidden;
                        let show_scrollbar = width_ratio < 1.0 && scrollbar_hidden;
                        if hide_scrollbar | show_scrollbar {
                            visibility_updated |= true;
                            scrollbars.scrollbar_h.update_layout(|layout| {
                                if hide_scrollbar {
                                    layout.hide();
                                }
                                if show_scrollbar {
                                    layout.show();
                                }
                            });
                        }
                    }

                    if height_ratio.is_finite() && (height_ratio - self.height_ratio).abs() > ::std::f32::EPSILON {
                        let height = self.container_rect.height() * height_ratio;
                        scrollbars.v_handle.update_layout(|layout| {
                            layout.edit_height().set(height);
                        });
                        let scrollbar_hidden = scrollbars.scrollbar_v.layout().hidden;
                        let hide_scrollbar = height_ratio >= 1.0 && !scrollbar_hidden;
                        let show_scrollbar = height_ratio < 1.0 && scrollbar_hidden;
                        if hide_scrollbar | show_scrollbar {
                            visibility_updated |= true;
                            scrollbars.scrollbar_v.update_layout(|layout| {
                                if hide_scrollbar {
                                    layout.hide();
                                }
                                if show_scrollbar {
                                    layout.show();
                                }
                            });
                        }
                    }

                    if visibility_updated {
                        if !scrollbars.scrollbar_h.layout().hidden && !scrollbars.scrollbar_v.layout().hidden {
                            scrollbars.corner.update_layout(|layout| layout.show());
                        } else {
                            scrollbars.corner.update_layout(|layout| layout.hide());
                        }
                    }
                }
                self.width_ratio = width_ratio;
                self.height_ratio = height_ratio;
            }
            ScrollParentEvent::WidgetMouseWheel(ref mouse_wheel) => {
                let scroll = get_scroll(mouse_wheel.0);
                if self.scrollable_area.width > 0.0 {
                    self.offset.x = f32::min(0.0, f32::max(-self.scrollable_area.width, self.offset.x + scroll.x));
                    self.move_content_x();
                    self.move_slider_x();
                }
                if self.scrollable_area.height > 0.0 {
                    self.offset.y = f32::min(0.0, f32::max(-self.scrollable_area.height, self.offset.y + scroll.y));
                    self.move_content_y();
                    self.move_slider_y();
                }
            }
            ScrollParentEvent::ScrollBarMovedX(ref offset) => {
                self.offset.x = -offset * self.scrollable_area.width;
                self.move_content_x();
            }
            ScrollParentEvent::ScrollBarMovedY(ref offset) => {
                self.offset.y = -offset * self.scrollable_area.height;
                self.move_content_y();
            }
        }
    }
}
fn get_scroll(event: glutin::MouseScrollDelta) -> Vector {
    let vec = match event {
        glutin::MouseScrollDelta::LineDelta(x, y) |
        glutin::MouseScrollDelta::PixelDelta(x, y) => {
            Vector::new(-x, y)
        }
    };
    vec * 13.0
}
