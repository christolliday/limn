use glutin;
use cassowary::strength::*;
use cassowary::WeightedRelation::*;

use event::{WidgetEventArgs, WidgetEventHandler};
use widget::{WidgetBuilder, WidgetBuilderCore, BuildWidget, WidgetRef};
use widgets::slider::{SliderBuilder, SetSliderValue};
use util::{Point, Size, Rect, RectExt};
use layout::{LayoutUpdated, LayoutVars, LayoutRef, LAYOUT};
use layout::constraint::*;
use input::mouse::WidgetMouseWheel;
use drawable::rect::{RectDrawable, RectStyleable};
use color::*;

pub struct ScrollBuilder {
    widget: WidgetBuilder,
    content_holder: WidgetBuilder,
    content: Option<WidgetBuilder>,
    scrollbars: Option<(WidgetBuilder, SliderBuilder, SliderBuilder)>,
}
impl ScrollBuilder {
    pub fn new() -> Self {
        let widget = WidgetBuilder::new_named("scroll");

        let mut content_holder = WidgetBuilder::new_named("content_holder");
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
    pub fn add_content<C: BuildWidget>(&mut self, mut widget: C) -> &mut Self {
        {
            let ref parent = self.content_holder.layout().vars;
            layout!(widget:
                LAYOUT.left | LE(REQUIRED) | parent.left,
                LAYOUT.top | LE(REQUIRED) | parent.top,
                LAYOUT.right | GE(REQUIRED) | parent.right,
                LAYOUT.bottom | GE(REQUIRED) | parent.bottom,
            );
        }
        self.content = Some(widget.build());
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

        let widget_ref = self.content_holder.widget.clone();
        scrollbar_h.on_value_changed(move |value, _| {
            widget_ref.event(ScrollParentEvent::OffsetX(value));
        });
        let widget_ref = self.content_holder.widget.clone();
        scrollbar_v.on_value_changed(move |value, _| {
            widget_ref.event(ScrollParentEvent::OffsetY(value));
        });
        let corner_style = style!(RectStyleable::BackgroundColor: MID_GRAY);
        let mut corner = WidgetBuilder::new_named("corner");
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
widget_builder!(ScrollBuilder, build: |mut builder: ScrollBuilder| -> WidgetBuilder {
    let widget_ref = builder.content_holder.widget.clone();
    builder.content_holder.add_handler_fn(move |_: &LayoutUpdated, _| {
        widget_ref.event(ScrollParentEvent::ContainerLayoutUpdated);
    });
    let mut content = builder.content.expect("Scroll bar has no content");
    let widget_ref = builder.content_holder.widget.clone();
    content.add_handler_fn(move |_: &LayoutUpdated, args| {
        widget_ref.event(ScrollParentEvent::ContentLayoutUpdated(args.widget.bounds()));
    });
    let mut scroll_parent_handler = ScrollParent::new(&mut content);
    if let Some((ref mut corner, ref mut scrollbar_h, ref mut scrollbar_v)) = builder.scrollbars {
        scroll_parent_handler.size_handler = Some(ScrollSizeHandler::new(scrollbar_h, scrollbar_v, corner.widget.clone()));
    }
    builder.content_holder.add_handler(scroll_parent_handler);
    builder.content_holder.add_handler_fn(|event: &WidgetMouseWheel, args| {
        args.widget.event(ScrollParentEvent::WidgetMouseWheel(event.clone()));
    });
    builder.content_holder.add_child(content);
    builder.widget.add_child(builder.content_holder);
    if let Some((corner, scrollbar_h, scrollbar_v)) = builder.scrollbars {
        builder.widget.add_child(corner);
        builder.widget.add_child(scrollbar_h);
        builder.widget.add_child(scrollbar_v);
    }
    builder.widget
});

#[allow(dead_code)]
struct ScrollSizeHandler {
    scrollbar_h_id: WidgetRef,
    scrollbar_v_id: WidgetRef,
    corner_id: WidgetRef,
    h_handle: WidgetRef,
    v_handle: WidgetRef,
}
impl ScrollSizeHandler {
    fn new(scrollbar_h: &mut SliderBuilder, scrollbar_v: &mut SliderBuilder, corner_id: WidgetRef) -> Self {
        ScrollSizeHandler {
            scrollbar_h_id: scrollbar_h.widget.widget.clone(),
            scrollbar_v_id: scrollbar_v.widget.widget.clone(),
            corner_id: corner_id,
            h_handle: scrollbar_h.slider_handle.widget.clone(),
            v_handle: scrollbar_v.slider_handle.widget.clone(),
        }
    }
}

enum ScrollParentEvent {
    ContainerLayoutUpdated,
    ContentLayoutUpdated(Rect),
    WidgetMouseWheel(WidgetMouseWheel),
    OffsetX(f64),
    OffsetY(f64),
}
struct ScrollParent {
    scrollable: WidgetRef,
    content_rect: Rect,
    width_ratio: f64,
    height_ratio: f64,
    scrollable_area: Size,
    offset: Point,
    pub size_handler: Option<ScrollSizeHandler>,
}
impl ScrollParent {
    fn new(scrollable: &mut WidgetBuilder) -> Self {
        ScrollParent {
            scrollable: scrollable.widget.clone(),
            content_rect: Rect::zero(),
            width_ratio: 0.0,
            height_ratio: 0.0,
            scrollable_area: Size::zero(),
            offset: Point::zero(),
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
                self.offset.x = f64::min(0.0, f64::max(max_scroll.x, self.offset.x));
                self.offset.y = f64::min(0.0, f64::max(max_scroll.y, self.offset.y));

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
fn get_scroll(event: glutin::MouseScrollDelta) -> Point {
    match event {
        glutin::MouseScrollDelta::LineDelta(x, y) => {
            Point::new(x as f64, y as f64)
        }
        glutin::MouseScrollDelta::PixelDelta(x, y) => {
            Point::new(x as f64, y as f64)
        }
    }
}
