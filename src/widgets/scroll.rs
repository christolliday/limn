use glutin;
use cassowary::strength::*;
use cassowary::WeightedRelation::*;

use event::{Target, WidgetEventArgs, WidgetEventHandler};
use widget::{Widget, WidgetBuilder, WidgetBuilderCore};
use util::{Point, Rectangle};
use layout::LAYOUT;
use layout::container::LayoutContainer;
use layout::solver::LimnSolver;
use resources::WidgetId;

use input::mouse::WidgetMouseWheel;

struct ScrollContainer;
impl LayoutContainer for ScrollContainer {
    fn set_padding(&mut self, _: f64) {}
    fn add_child(&mut self, parent: &Widget, child: &mut WidgetBuilder) {
        let ref parent = parent.layout;
        layout!(child:
            LAYOUT.left | LE(REQUIRED) | parent.left,
            LAYOUT.top | LE(REQUIRED) | parent.top,
            // STRONG not REQUIRED because not satisfiable if layout is smaller than it's parent
            LAYOUT.right | GE(STRONG) | parent.right,
            LAYOUT.bottom | GE(STRONG) | parent.bottom);
    }
    fn remove_child(&mut self, _: &Widget, _: WidgetId, _: &mut LimnSolver) {}
}

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

pub struct WidgetScrollHandler {
    offset: Point,
}
impl WidgetScrollHandler {
    pub fn new() -> Self {
        WidgetScrollHandler { offset: Point { x: 0.0, y: 0.0 } }
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
        let &WidgetScroll { event, parent_bounds } = event;
        let scroll = get_scroll(event);
        let widget_bounds = args.widget.layout.bounds();

        let max_scroll = Point::new(
            parent_bounds.width - widget_bounds.width,
            parent_bounds.height - widget_bounds.height);
        self.offset = self.offset + scroll * 13.0;
        self.offset.x = f64::min(0.0, f64::max(max_scroll.x, self.offset.x));
        self.offset.y = f64::min(0.0, f64::max(max_scroll.y, self.offset.y));
        args.widget.update_layout(|layout| {
            layout.edit_left().set(parent_bounds.left + self.offset.x);
            layout.edit_top().set(parent_bounds.top + self.offset.y);
        }, args.solver);
    }
}

impl WidgetBuilder {
    pub fn contents_scroll(&mut self) -> &mut Self {
        self.set_container(ScrollContainer);
        self.add_handler_fn(scroll_handle_mouse_wheel)
    }
    pub fn make_scrollable(&mut self) -> &mut Self {
        self.add_handler(WidgetScrollHandler::new())
    }
}