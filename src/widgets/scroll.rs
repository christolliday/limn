use glutin;

use event::{Target, WidgetEventArgs, WidgetEventHandler};
use widget::{WidgetBuilder, WidgetBuilderCore};
use ui::ChildAttachedEvent;
use util::{Point, Rectangle};

use input::mouse::WidgetMouseWheel;

pub struct WidgetScroll {
    event: glutin::MouseScrollDelta,
    parent_bounds: Rectangle,
}

fn scroll_handle_mouse_wheel(event: &WidgetMouseWheel, args: WidgetEventArgs) {
    let WidgetEventArgs { widget, queue, .. } = args;
    let widget_bounds = widget.layout.bounds();
    let event = WidgetScroll {
        event: event.0,
        parent_bounds: widget_bounds,
    };
    queue.push(Target::Child(widget.id), event);
}

fn scroll_handle_child_added(event: &ChildAttachedEvent, args: WidgetEventArgs) {
    let &ChildAttachedEvent(_, ref child_layout) = event;
    args.widget.update_layout(|layout| {
        layout.scroll_parent(child_layout);
    }, args.solver);
}

pub struct WidgetScrollHandler {
    offset: Point,
}
impl WidgetScrollHandler {
    pub fn new() -> Self {
        WidgetScrollHandler { offset: Point { x: 0.0, y: 0.0 } }
    }
}
impl WidgetEventHandler<WidgetScroll> for WidgetScrollHandler {
    fn handle(&mut self, event: &WidgetScroll, args: WidgetEventArgs) {
        let &WidgetScroll { event, parent_bounds } = event;

        let scroll = match event {
            glutin::MouseScrollDelta::LineDelta(x, y) => {
                Point {
                    x: x as f64,
                    y: y as f64,
                }
            }
            glutin::MouseScrollDelta::PixelDelta(x, y) => {
                Point {
                    x: x as f64,
                    y: y as f64,
                }
            }
        };

        let widget_bounds = args.widget.layout.bounds();

        self.offset = self.offset + scroll * 13.0;
        self.offset.x = f64::min(0.0,
                                 f64::max(parent_bounds.width - widget_bounds.width,
                                          self.offset.x));
        self.offset.y = f64::min(0.0,
                                 f64::max(parent_bounds.height - widget_bounds.height,
                                          self.offset.y));
        args.widget.update_layout(|layout| {
            layout.edit_left().set(parent_bounds.left + self.offset.x);
            layout.edit_top().set(parent_bounds.top + self.offset.y);
        }, args.solver);
    }
}

impl WidgetBuilder {
    pub fn contents_scroll(&mut self) -> &mut Self {
        self.bound_children = false;
        self.add_handler_fn(scroll_handle_child_added);
        self.add_handler_fn(scroll_handle_mouse_wheel)
    }
    pub fn make_scrollable(&mut self) -> &mut Self {
        self.add_handler(WidgetScrollHandler::new())
    }
}