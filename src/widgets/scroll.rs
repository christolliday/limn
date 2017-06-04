use glutin;
use cassowary::Variable;
use cassowary::strength::*;

use event::{Target, WidgetEventArgs, WidgetEventHandler};
use widget::{WidgetBuilder, WidgetBuilderCore, BuildWidget};
use widgets::slider::SliderBuilder;
use util::{Point, Size, Rect, RectExt};
use layout::{LayoutUpdated, LayoutVars, LayoutRef};
use layout::constraint::*;
use resources::WidgetId;
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
        // only used to set initial position
        layout!(widget:
            align_left(&self.content_holder).strength(WEAK),
            align_top(&self.content_holder).strength(WEAK),
        );
        widget.add_handler(WidgetScrollHandler::new());
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

        let widget_id = self.content_holder.id();
        scrollbar_h.on_value_changed(move |value, _| {
            let event = ScrollParentEvent::OffsetX(value);
            event!(Target::Widget(widget_id), event);
        });
        scrollbar_v.on_value_changed(move |value, _| {
            let event = ScrollParentEvent::OffsetY(value);
            event!(Target::Widget(widget_id), event);
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
    let widget_id = builder.content_holder.id();
    builder.content_holder.add_handler_fn(move |_: &LayoutUpdated, _| {
        event!(Target::Widget(widget_id), ScrollParentEvent::ContainerSizeChange);
    });
    let mut content = builder.content.expect("Scroll bar has no content");
    content.add_handler_fn(move |_: &LayoutUpdated, args| {
        event!(Target::Widget(widget_id), ScrollParentEvent::ContentSizeChange(args.widget.bounds.size));
    });
    let mut scroll_parent_handler = ScrollParent::new(content.id());
    if let Some((_, ref mut scrollbar_h, ref mut scrollbar_v)) = builder.scrollbars {
        let h_handle_size = scrollbar_h.slider_handle.layout().vars.width;
        let v_handle_size = scrollbar_v.slider_handle.layout().vars.height;
        scroll_parent_handler.size_handler = Some(ScrollSizeHandler::new(h_handle_size, v_handle_size));
    }
    builder.content_holder.add_handler(scroll_parent_handler);
    builder.content_holder.add_handler_fn(|event: &WidgetMouseWheel, args| {
        event!(Target::Widget(args.widget.id), ScrollParentEvent::WidgetMouseWheel(event.clone()));
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

struct ScrollSizeHandler {
    content_size: Size,
    h_handle_size: Variable,
    v_handle_size: Variable,
}
impl ScrollSizeHandler {
    fn new(h_handle_size: Variable, v_handle_size: Variable) -> Self {
        ScrollSizeHandler {
            content_size: Size::zero(),
            h_handle_size: h_handle_size,
            v_handle_size: v_handle_size,
        }
    }
}

enum ScrollParentEvent {
    ContainerSizeChange,
    ContentSizeChange(Size),
    WidgetMouseWheel(WidgetMouseWheel),
    OffsetX(f64),
    OffsetY(f64),
}
struct ScrollParent {
    scrollable: WidgetId,
    pub size_handler: Option<ScrollSizeHandler>,
}
impl ScrollParent {
    fn new(scrollable: WidgetId) -> Self {
        ScrollParent {
            scrollable: scrollable,
            size_handler: None,
        }
    }
}
impl WidgetEventHandler<ScrollParentEvent> for ScrollParent {
    fn handle(&mut self, event: &ScrollParentEvent, args: WidgetEventArgs) {
        match *event {
            ScrollParentEvent::ContainerSizeChange | ScrollParentEvent::ContentSizeChange(_) => {
                if let Some(ref mut size_handler) = self.size_handler {
                    let container_size = args.widget.bounds;
                    let old_width_ratio = container_size.width() / size_handler.content_size.width;
                    let old_height_ratio = container_size.height() / size_handler.content_size.height;
                    match event {
                        &ScrollParentEvent::ContentSizeChange(size) => size_handler.content_size = size,
                        _ => (),
                    }
                    let width_ratio = container_size.width() / size_handler.content_size.width;
                    let height_ratio = container_size.height() / size_handler.content_size.height;
                    if width_ratio.is_finite() && width_ratio != old_width_ratio {
                        let width = container_size.width() * width_ratio;
                        debug!("width_ratio {:?} {:?}", width_ratio, width);
                        args.solver.update_solver(|solver| {
                            solver.suggest_value(size_handler.h_handle_size, width).unwrap();
                        });
                    }
                    if height_ratio.is_finite() && height_ratio != old_height_ratio {
                        let height = container_size.height() * height_ratio;
                        debug!("height_ratio {:?} {:?}", height_ratio, height);
                        args.solver.update_solver(|solver| {
                            solver.suggest_value(size_handler.v_handle_size, height).unwrap();
                        });
                    }
                }
            }
            ScrollParentEvent::WidgetMouseWheel(ref mouse_wheel) => {
                let event = WidgetScroll::MouseWheel(mouse_wheel.0, args.widget.bounds);
                event!(Target::Widget(self.scrollable), event);
            }
            ScrollParentEvent::OffsetX(ref offset) => {
                if let Some(ref mut size_handler) = self.size_handler {
                    let width_ratio = size_handler.content_size.width - args.widget.bounds.width();
                    let event = WidgetScroll::OffsetX(-offset * width_ratio, args.widget.bounds);
                    event!(Target::Widget(self.scrollable), event);
                }
            }
            ScrollParentEvent::OffsetY(ref offset) => {
                if let Some(ref mut size_handler) = self.size_handler {
                    let height_ratio = size_handler.content_size.height - args.widget.bounds.height();
                    let event = WidgetScroll::OffsetY(-offset * height_ratio, args.widget.bounds);
                    event!(Target::Widget(self.scrollable), event);
                }
            }
        }
    }
}

pub enum WidgetScroll {
    OffsetX(f64, Rect),
    OffsetY(f64, Rect),
    MouseWheel(glutin::MouseScrollDelta, Rect),
}

pub struct WidgetScrollHandler {
    offset: Point,
}
impl WidgetScrollHandler {
    pub fn new() -> Self {
        WidgetScrollHandler { offset: Point::zero() }
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
impl WidgetEventHandler<WidgetScroll> for WidgetScrollHandler {
    fn handle(&mut self, event: &WidgetScroll, args: WidgetEventArgs) {
        match *event {
            WidgetScroll::MouseWheel(event, parent_bounds) => {
                let scroll = get_scroll(event);
                let widget_bounds = args.widget.bounds;

                let max_scroll = Point::new(
                    parent_bounds.width() - widget_bounds.width(),
                    parent_bounds.height() - widget_bounds.height());
                self.offset = self.offset + scroll * 13.0;
                self.offset.x = f64::min(0.0, f64::max(max_scroll.x, self.offset.x));
                self.offset.y = f64::min(0.0, f64::max(max_scroll.y, self.offset.y));
                args.widget.update_layout(|layout| {
                    layout.edit_left().set(parent_bounds.left() + self.offset.x);
                    layout.edit_top().set(parent_bounds.top() + self.offset.y);
                }, args.solver);
            }
            WidgetScroll::OffsetX(offset, parent_bounds) => {
                self.offset.x = offset;
                args.widget.update_layout(|layout| {
                    layout.edit_left().set(parent_bounds.left() + self.offset.x);
                }, args.solver);
            }
            WidgetScroll::OffsetY(offset, parent_bounds) => {
                self.offset.y = offset;
                args.widget.update_layout(|layout| {
                    layout.edit_top().set(parent_bounds.top() + self.offset.y);
                }, args.solver);
            }
        }
    }
}
