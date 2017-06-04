use glutin;
use cassowary::Variable;
use cassowary::strength::*;

use event::{Target, WidgetEventArgs, WidgetEventHandler};
use widget::{WidgetBuilder, WidgetBuilderCore, BuildWidget};
use widgets::slider::{SliderBuilder, SetSliderValue};
use util::{Point, Size, RectExt};
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
    let mut scroll_parent_handler = ScrollParent::new(&mut content);
    if let Some((_, ref mut scrollbar_h, ref mut scrollbar_v)) = builder.scrollbars {
        scroll_parent_handler.size_handler = Some(ScrollSizeHandler::new(scrollbar_h, scrollbar_v));
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
    scrollbar_h_id: WidgetId,
    scrollbar_v_id: WidgetId,
    h_handle_size: Variable,
    v_handle_size: Variable,
}
impl ScrollSizeHandler {
    fn new(scrollbar_h: &mut SliderBuilder, scrollbar_v: &mut SliderBuilder) -> Self {
        ScrollSizeHandler {
            scrollbar_h_id: scrollbar_h.id(),
            scrollbar_v_id: scrollbar_v.id(),
            h_handle_size: scrollbar_h.slider_handle.layout().vars.width,
            v_handle_size: scrollbar_v.slider_handle.layout().vars.height,
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
    scrollable_left: Variable,
    scrollable_top: Variable,
    content_size: Size,
    offset: Point,
    pub size_handler: Option<ScrollSizeHandler>,
}
impl ScrollParent {
    fn new(scrollable: &mut WidgetBuilder) -> Self {
        ScrollParent {
            scrollable_left: scrollable.layout().vars.left,
            scrollable_top: scrollable.layout().vars.top,
            content_size: Size::zero(),
            offset: Point::zero(),
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
                    let old_width_ratio = container_size.width() / self.content_size.width;
                    let old_height_ratio = container_size.height() / self.content_size.height;

                    if let &ScrollParentEvent::ContentSizeChange(size) = event {
                        self.content_size = size
                    }
                    let width_ratio = container_size.width() / self.content_size.width;
                    let height_ratio = container_size.height() / self.content_size.height;
                    if width_ratio.is_finite() && width_ratio != old_width_ratio {
                        let width = container_size.width() * width_ratio;
                        args.solver.update_solver(|solver| {
                            solver.suggest_value(size_handler.h_handle_size, width).unwrap();
                        });
                    }
                    if height_ratio.is_finite() && height_ratio != old_height_ratio {
                        let height = container_size.height() * height_ratio;
                        args.solver.update_solver(|solver| {
                            solver.suggest_value(size_handler.v_handle_size, height).unwrap();
                        });
                    }
                } else if let &ScrollParentEvent::ContentSizeChange(size) = event {
                    self.content_size = size;
                }
            }
            ScrollParentEvent::WidgetMouseWheel(ref mouse_wheel) => {
                let scroll = get_scroll(mouse_wheel.0);
                let parent_bounds = args.widget.bounds;
                let widget_bounds = self.content_size;

                let max_scroll = Point::new(
                    parent_bounds.width() - widget_bounds.width,
                    parent_bounds.height() - widget_bounds.height);
                self.offset = self.offset + scroll * 13.0;
                self.offset.x = f64::min(0.0, f64::max(max_scroll.x, self.offset.x));
                self.offset.y = f64::min(0.0, f64::max(max_scroll.y, self.offset.y));

                args.solver.update_solver(|solver| {
                    if !solver.has_edit_variable(&self.scrollable_left) {
                        solver.add_edit_variable(self.scrollable_left, STRONG).unwrap();
                        solver.add_edit_variable(self.scrollable_top, STRONG).unwrap();
                    }
                    solver.suggest_value(self.scrollable_left, parent_bounds.left() + self.offset.x).unwrap();
                    solver.suggest_value(self.scrollable_top, parent_bounds.top() + self.offset.y).unwrap();
                });

                if let Some(ref mut size_handler) = self.size_handler {
                    let offset_x = -self.offset.x / (self.content_size.width - args.widget.bounds.width());
                    let offset_y = -self.offset.y / (self.content_size.height - args.widget.bounds.height());
                    event!(Target::Widget(size_handler.scrollbar_h_id), SetSliderValue(offset_x));
                    event!(Target::Widget(size_handler.scrollbar_v_id), SetSliderValue(offset_y));
                }
            }
            ScrollParentEvent::OffsetX(ref offset) => {
                self.offset.x = -offset * (self.content_size.width - args.widget.bounds.width());
                let parent_bounds = args.widget.bounds;
                args.solver.update_solver(|solver| {
                    if !solver.has_edit_variable(&self.scrollable_left) {
                        solver.add_edit_variable(self.scrollable_left, STRONG).unwrap();
                    }
                    solver.suggest_value(self.scrollable_left, parent_bounds.left() + self.offset.x).unwrap();
                });
            }
            ScrollParentEvent::OffsetY(ref offset) => {
                self.offset.y = -offset * (self.content_size.height - args.widget.bounds.height());
                let parent_bounds = args.widget.bounds;
                args.solver.update_solver(|solver| {
                    if !solver.has_edit_variable(&self.scrollable_top) {
                        solver.add_edit_variable(self.scrollable_top, STRONG).unwrap();
                    }
                    solver.suggest_value(self.scrollable_top, parent_bounds.top() + self.offset.y).unwrap();
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
