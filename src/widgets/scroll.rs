use glutin;
use cassowary::strength::*;
use cassowary::WeightedRelation::*;

use event::{WidgetEventArgs, WidgetEventHandler};
use widget::{BuildWidget, Widget};
use widgets::slider::{SliderBuilder, SetSliderValue};
use util::{Point, Size, Vector, Rect, RectExt};
use layout::{LayoutUpdated, LAYOUT};
use input::mouse::WidgetMouseWheel;
use drawable::rect::{RectDrawable, RectStyleable};
use color::*;

pub struct ScrollBuilder {
    widget: Widget,
    content_holder: Widget,
    content: Option<Widget>,
    scrollbars: Option<(Widget, SliderBuilder, SliderBuilder)>,
}
impl ScrollBuilder {
    pub fn new() -> Self {
        let widget = Widget::new_named("scroll");

        let mut content_holder = Widget::new_named("content_holder");
        content_holder.no_container();
        layout!(content_holder:
            align_left(&widget),
            align_top(&widget));

        ScrollBuilder {
            widget: widget,
            content_holder: content_holder,
            content: None,
            scrollbars: None,
        }
    }
    pub fn add_content<C: BuildWidget>(&mut self, widget: C) -> &mut Self {
        let mut widget = widget.build();
        {
            let ref parent = self.content_holder.layout().vars;
            layout!(widget:
                LAYOUT.left | LE(REQUIRED) | parent.left,
                LAYOUT.top | LE(REQUIRED) | parent.top,
                LAYOUT.right | GE(REQUIRED) | parent.right,
                LAYOUT.bottom | GE(REQUIRED) | parent.bottom,
            );
        }
        self.content = Some(widget);
        self
    }
    pub fn add_scrollbar(&mut self) -> &mut Self {
        let mut scrollbar_h = SliderBuilder::new();
        scrollbar_h.set_debug_name("scrollbar_h");
        scrollbar_h.scrollbar_style();
        layout!(scrollbar_h:
            align_bottom(&self.widget),
            align_left(&self.widget),
            below(&self.content_holder),
        );
        let mut scrollbar_v = SliderBuilder::new();
        scrollbar_v.set_debug_name("scrollbar_v");
        scrollbar_v.make_vertical().scrollbar_style();
        layout!(scrollbar_v:
            align_right(&self.widget),
            align_top(&self.widget),
            to_right_of(&self.content_holder),
        );

        let widget_ref = self.content_holder.clone();
        scrollbar_h.on_value_changed(move |value, _| {
            widget_ref.event(ScrollParentEvent::OffsetX(value));
        });
        let widget_ref = self.content_holder.clone();
        scrollbar_v.on_value_changed(move |value, _| {
            widget_ref.event(ScrollParentEvent::OffsetY(value));
        });
        let corner_style = style!(RectStyleable::BackgroundColor: GRAY_70);
        let mut corner = Widget::new_named("corner");
        corner.set_drawable_with_style(RectDrawable::new(), corner_style);
        layout!(corner:
            align_bottom(&self.widget),
            align_right(&self.widget),
            to_right_of(&scrollbar_h),
            below(&scrollbar_v),
            match_height(&scrollbar_h),
            match_width(&scrollbar_v),
        );

        self.scrollbars = Some((corner, scrollbar_h, scrollbar_v));
        self
    }
}
impl BuildWidget for ScrollBuilder {
    fn build(mut self) -> Widget {
        let widget_ref = self.content_holder.clone();
        self.content_holder.add_handler_fn(move |_: &LayoutUpdated, _| {
            widget_ref.event(ScrollParentEvent::ContainerLayoutUpdated);
        });
        let mut content = self.content.expect("Scroll bar has no content");
        let widget_ref = self.content_holder.clone();
        content.add_handler_fn(move |_: &LayoutUpdated, args| {
            widget_ref.event(ScrollParentEvent::ContentLayoutUpdated(args.widget.bounds()));
        });
        let mut scroll_parent_handler = ScrollParent::new(&mut content);
        if let Some((ref mut corner, ref mut scrollbar_h, ref mut scrollbar_v)) = self.scrollbars {
            scroll_parent_handler.size_handler = Some(ScrollSizeHandler::new(scrollbar_h, scrollbar_v, corner.clone()));
        }
        self.content_holder.add_handler(scroll_parent_handler);
        self.content_holder.add_handler_fn(|event: &WidgetMouseWheel, args| {
            args.widget.event(ScrollParentEvent::WidgetMouseWheel(event.clone()));
        });
        self.content_holder.add_child(content);
        self.widget.add_child(self.content_holder);
        if let Some((corner, scrollbar_h, scrollbar_v)) = self.scrollbars {
            self.widget.add_child(corner);
            self.widget.add_child(scrollbar_h.build());
            self.widget.add_child(scrollbar_v.build());
        }
        self.widget
    }
}
widget_builder!(ScrollBuilder);

#[allow(dead_code)]
struct ScrollSizeHandler {
    scrollbar_h_id: Widget,
    scrollbar_v_id: Widget,
    corner_id: Widget,
    h_handle: Widget,
    v_handle: Widget,
}
impl ScrollSizeHandler {
    fn new(scrollbar_h: &mut SliderBuilder, scrollbar_v: &mut SliderBuilder, corner_id: Widget) -> Self {
        ScrollSizeHandler {
            scrollbar_h_id: scrollbar_h.widget.clone(),
            scrollbar_v_id: scrollbar_v.widget.clone(),
            corner_id: corner_id,
            h_handle: scrollbar_h.slider_handle.clone(),
            v_handle: scrollbar_v.slider_handle.clone(),
        }
    }
}

enum ScrollParentEvent {
    ContainerLayoutUpdated,
    ContentLayoutUpdated(Rect),
    WidgetMouseWheel(WidgetMouseWheel),
    OffsetX(f32),
    OffsetY(f32),
}
struct ScrollParent {
    scrollable: Widget,
    content_rect: Rect,
    width_ratio: f32,
    height_ratio: f32,
    scrollable_area: Size,
    offset: Vector,
    pub size_handler: Option<ScrollSizeHandler>,
}
impl ScrollParent {
    fn new(scrollable: &mut Widget) -> Self {
        ScrollParent {
            scrollable: scrollable.clone(),
            content_rect: Rect::zero(),
            width_ratio: 0.0,
            height_ratio: 0.0,
            scrollable_area: Size::zero(),
            offset: Vector::zero(),
            size_handler: None,
        }
    }
}
impl WidgetEventHandler<ScrollParentEvent> for ScrollParent {
    fn handle(&mut self, event: &ScrollParentEvent, args: WidgetEventArgs) {
        match *event {
            ScrollParentEvent::ContainerLayoutUpdated | ScrollParentEvent::ContentLayoutUpdated(_) => {
                if let &ScrollParentEvent::ContentLayoutUpdated(rect) = event {
                    self.content_rect = rect
                }
                if let Some(ref mut size_handler) = self.size_handler {
                    let container_size = args.widget.bounds().size;
                    let width_ratio = container_size.width / self.content_rect.size.width;
                    let height_ratio = container_size.height / self.content_rect.size.height;
                    let content_offset = self.content_rect.origin - args.widget.bounds().origin;
                    if width_ratio.is_finite() && width_ratio != self.width_ratio {
                        self.width_ratio = width_ratio;
                        let width = container_size.width * width_ratio;
                        size_handler.h_handle.update_layout(|layout| {
                            layout.edit_width().set(width);
                        });
                    }
                    if height_ratio.is_finite() && height_ratio != self.height_ratio {
                        self.height_ratio = height_ratio;
                        let height = container_size.height * height_ratio;
                        size_handler.v_handle.update_layout(|layout| {
                            layout.edit_height().set(height);
                        });
                    }
                    let scrollable_area = self.content_rect.size - args.widget.bounds().size;
                    if content_offset != self.offset || scrollable_area != self.scrollable_area {
                        self.offset = content_offset;
                        self.scrollable_area = scrollable_area;

                        if scrollable_area.width > 0.0 {
                            let offset_x = -content_offset.x / scrollable_area.width;
                            size_handler.scrollbar_h_id.event(SetSliderValue(offset_x));
                        }
                        if scrollable_area.height > 0.0 {
                            let offset_y = -content_offset.y / scrollable_area.height;
                            size_handler.scrollbar_v_id.event(SetSliderValue(offset_y));
                        }
                    }
                }
            }
            ScrollParentEvent::WidgetMouseWheel(ref mouse_wheel) => {
                let scroll = get_scroll(mouse_wheel.0);
                let parent_bounds = args.widget.bounds();

                let max_scroll = Point::new(
                    parent_bounds.width() - self.content_rect.width(),
                    parent_bounds.height() - self.content_rect.height());
                self.offset = self.offset + scroll * 13.0;
                self.offset.x = f32::min(0.0, f32::max(max_scroll.x, self.offset.x));
                self.offset.y = f32::min(0.0, f32::max(max_scroll.y, self.offset.y));

                let scrollable_left = parent_bounds.left() + self.offset.x;
                let scrollable_top = parent_bounds.top() + self.offset.y;
                self.scrollable.update_layout(|layout| {
                    layout.edit_left().set(scrollable_left);
                    layout.edit_top().set(scrollable_top);
                });

                if let Some(ref mut size_handler) = self.size_handler {
                    let scrollable_area = self.content_rect.size - args.widget.bounds().size;
                    if scrollable_area.width > 0.0 {
                        let offset_x = -self.offset.x / scrollable_area.width;
                        size_handler.scrollbar_h_id.event(SetSliderValue(offset_x));
                    }
                    if scrollable_area.height > 0.0 {
                        let offset_y = -self.offset.y / scrollable_area.height;
                        size_handler.scrollbar_v_id.event(SetSliderValue(offset_y));
                    }
                }
            }
            ScrollParentEvent::OffsetX(ref offset) => {
                self.offset.x = -offset * (self.content_rect.width() - args.widget.bounds().width());
                let parent_bounds = args.widget.bounds();
                let scrollable_left = parent_bounds.left() + self.offset.x;
                self.scrollable.update_layout(|layout| {
                    layout.edit_left().set(scrollable_left);
                });
            }
            ScrollParentEvent::OffsetY(ref offset) => {
                self.offset.y = -offset * (self.content_rect.height() - args.widget.bounds().height());
                let parent_bounds = args.widget.bounds();
                let scrollable_top = parent_bounds.top() + self.offset.y;
                self.scrollable.update_layout(|layout| {
                    layout.edit_top().set(scrollable_top);
                });
            }
        }
    }
}
fn get_scroll(event: glutin::MouseScrollDelta) -> Vector {
    match event {
        glutin::MouseScrollDelta::LineDelta(x, y) => {
            Vector::new(x as f32, y as f32)
        }
        glutin::MouseScrollDelta::PixelDelta(x, y) => {
            Vector::new(x as f32, y as f32)
        }
    }
}
