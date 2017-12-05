use glutin;
use cassowary::strength::*;
use cassowary::WeightedRelation::*;

use layout::constraint::*;
use event::{EventArgs, EventHandler};
use widget::Widget;
use widgets::slider::{SliderStyle, SliderEvent, SetSliderValue, Orientation};
use geometry::{Size, Vector, Rect, RectExt};
use layout::{LayoutUpdated, LAYOUT};
use input::mouse::WidgetMouseWheel;
use draw::rect::RectStyle;
use color::*;
use style::WidgetModifier;

component_style!{pub struct ScrollContainer<name="scroll", style=ScrollStyle> {
    content: Option<Widget> = None,
    has_scrollbars: bool = false,
}}

impl ScrollContainer {
    /// Set the scrollable content
    pub fn add_content(&mut self, widget: Widget) -> &mut Self {
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
    fn apply(&self, widget: &mut Widget) {
        let mut content_holder = Widget::new("content_holder");
        content_holder.layout().no_container();
        let mut content = self.content.clone().expect("Scroll bar has no content");
        forward_event!(LayoutUpdated: |_, args| ContentLayoutUpdated(args.widget.bounds()); content -> content_holder);
        content_holder.layout().add(constraints![
            match_layout(widget).strength(STRONG)
        ]);
        {
            let content_holder = &content_holder.layout().vars;
            content.layout().add(constraints![
                LAYOUT.left | LE(REQUIRED) | content_holder.left,
                LAYOUT.top | LE(REQUIRED) | content_holder.top,
                LAYOUT.left | EQ(WEAK) | content_holder.left,
                LAYOUT.top | EQ(WEAK) | content_holder.top,
                LAYOUT.right | GE(STRONG) | content_holder.right,
                LAYOUT.bottom | GE(STRONG) | content_holder.bottom,
            ]);
        }
        let mut scrollbars = if self.has_scrollbars {
            let mut scrollbar_h = Widget::from_modifier_style_class(SliderStyle::default(), "scrollbar_slider");
            scrollbar_h.set_name("scrollbar_h");
            scrollbar_h.layout().add(constraints![
                align_bottom(widget),
                align_left(widget),
                align_below(&content_holder),
            ]);
            let mut scrollbar_v = Widget::from_modifier_style_class(
                style!(SliderStyle { orientation: Orientation::Vertical, }), "scrollbar_slider");
            scrollbar_v.set_name("scrollbar_v");
            scrollbar_v.layout().add(constraints![
                align_right(widget),
                align_top(widget),
                align_to_right_of(&content_holder),
            ]);
            let mut corner = Widget::new("corner");
            corner.set_draw_style(style!(RectStyle { background_color: GRAY_70, }));
            corner.layout().add(constraints![
                align_bottom(widget),
                align_right(widget),
                align_to_right_of(&scrollbar_h),
                align_below(&scrollbar_v),
                match_height(&scrollbar_h),
                match_width(&scrollbar_v),
            ]);

            forward_event!(SliderEvent: |event, _| ScrollBarMoved::Horizontal(event.value); scrollbar_h -> content_holder);
            forward_event!(SliderEvent: |event, _| ScrollBarMoved::Vertical(event.value); scrollbar_v -> content_holder);

            Some((corner, scrollbar_h, scrollbar_v))
        } else {
            None
        };

        let mut scroll_parent_handler = ScrollParent::new(&mut content);
        if let Some((ref mut corner, ref mut scrollbar_h, ref mut scrollbar_v)) = scrollbars {
            scroll_parent_handler.scrollbars = Some(ScrollBars::new(scrollbar_h.clone(), scrollbar_v.clone(), corner.clone()));
        }
        content_holder.add_handler(scroll_parent_handler);
        ScrollParent::add_adapters(&mut content_holder);

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
    scrollbar_h: Widget,
    scrollbar_v: Widget,
    corner: Widget,
    h_handle: Widget,
    v_handle: Widget,
}
impl ScrollBars {
    fn new(scrollbar_h: Widget, scrollbar_v: Widget, corner: Widget) -> Self {
        let h_handle = scrollbar_h.child("slider_handle").unwrap();
        let v_handle = scrollbar_v.child("slider_handle").unwrap();
        ScrollBars {
            scrollbar_h: scrollbar_h,
            scrollbar_v: scrollbar_v,
            corner: corner,
            h_handle: h_handle,
            v_handle: v_handle,
        }
    }
}

#[derive(Clone)]
struct ContentLayoutUpdated(Rect);
#[derive(Clone)]
enum ScrollBarMoved {
    Horizontal(f32),
    Vertical(f32),
}

multi_event!{impl EventHandler<ScrollParentEvent> for ScrollParent {
    LayoutUpdated => container_layout_updated,
    ContentLayoutUpdated => content_layout_updated,
    WidgetMouseWheel => widget_mouse_wheel,
    ScrollBarMoved => scrollbar_moved,
}}

struct ScrollParent {
    scrollable: Widget,
    content_rect: Rect,
    container_rect: Rect,
    width_ratio: f32,
    height_ratio: f32,
    scrollable_area: Size,
    offset: Vector,
    pub scrollbars: Option<ScrollBars>,
}

impl ScrollParent {
    fn new(scrollable: &mut Widget) -> Self {
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

    fn container_layout_updated(&mut self, _: &LayoutUpdated, args: EventArgs) {
        self.container_rect = args.widget.bounds();
        self.update_bounds();
    }

    fn content_layout_updated(&mut self, event: &ContentLayoutUpdated, _: EventArgs) {
        self.content_rect = event.0;
        self.update_bounds();
    }

    fn update_bounds(&mut self) {
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
                let mut layout = scrollbars.h_handle.layout();
                layout.edit_width().set(width);
                let mut layout = scrollbars.scrollbar_h.layout();
                let scrollbar_hidden = layout.hidden;
                let hide_scrollbar = width_ratio >= 1.0 && !scrollbar_hidden;
                let show_scrollbar = width_ratio < 1.0 && scrollbar_hidden;
                if hide_scrollbar | show_scrollbar {
                    visibility_updated |= true;
                    if hide_scrollbar {
                        layout.hide();
                    }
                    if show_scrollbar {
                        layout.show();
                    }
                }
            }

            if height_ratio.is_finite() && (height_ratio - self.height_ratio).abs() > ::std::f32::EPSILON {
                let height = self.container_rect.height() * height_ratio;
                let mut layout = scrollbars.v_handle.layout();
                layout.edit_height().set(height);
                let scrollbar_hidden = layout.hidden;
                let hide_scrollbar = height_ratio >= 1.0 && !scrollbar_hidden;
                let show_scrollbar = height_ratio < 1.0 && scrollbar_hidden;
                if hide_scrollbar | show_scrollbar {
                    visibility_updated |= true;
                    let mut layout = scrollbars.scrollbar_v.layout();
                    if hide_scrollbar {
                        layout.hide();
                    }
                    if show_scrollbar {
                        layout.show();
                    }
                }
            }

            if visibility_updated {
                if !scrollbars.scrollbar_h.layout().hidden && !scrollbars.scrollbar_v.layout().hidden {
                    scrollbars.corner.layout().show();
                } else {
                    scrollbars.corner.layout().hide();
                }
            }
        }
        self.width_ratio = width_ratio;
        self.height_ratio = height_ratio;
    }

    fn widget_mouse_wheel(&mut self, event: &WidgetMouseWheel, _: EventArgs) {
        let &WidgetMouseWheel(mouse_wheel) = event;
        let scroll = get_scroll(mouse_wheel);
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

    fn scrollbar_moved(&mut self, event: &ScrollBarMoved, _: EventArgs) {
        match *event {
            ScrollBarMoved::Horizontal(offset) => {
                self.offset.x = -offset * self.scrollable_area.width;
                self.move_content_x();
            }
            ScrollBarMoved::Vertical(offset) => {
                self.offset.y = -offset * self.scrollable_area.height;
                self.move_content_y();
            }
        }
    }

    fn move_content_x(&mut self) {
        let scroll_to = self.container_rect.left() + self.offset.x;
        let mut layout = self.scrollable.layout();
        layout.edit_left().set(scroll_to);
    }
    fn move_content_y(&mut self) {
        let scroll_to = self.container_rect.top() + self.offset.y;
        let mut layout = self.scrollable.layout();
        layout.edit_top().set(scroll_to);
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

fn get_scroll(event: glutin::MouseScrollDelta) -> Vector {
    let vec = match event {
        glutin::MouseScrollDelta::LineDelta(x, y) |
        glutin::MouseScrollDelta::PixelDelta(x, y) => {
            Vector::new(-x, y)
        }
    };
    vec * 13.0
}
