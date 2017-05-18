use glutin;
use cassowary::strength::*;

use event::{Target, WidgetEventArgs, WidgetEventHandler};
use widget::{Widget, WidgetBuilder, WidgetBuilderCore, BuildWidget};
use util::{Point, Rect, RectExt};
use layout::solver::LimnSolver;
use layout::container::LayoutContainer;
use layout::constraint::*;
use resources::WidgetId;
use input::mouse::WidgetMouseWheel;

pub struct ScrollBuilder {
    widget: WidgetBuilder,
    content_holder: WidgetBuilder,
}
impl ScrollBuilder {
    pub fn new() -> Self {
        let widget = WidgetBuilder::new();

        let mut content_holder = WidgetBuilder::new();
        content_holder.set_container(ScrollContainer);
        content_holder.add_handler(ScrollParent::new());
        content_holder.add_handler_fn(|event: &WidgetMouseWheel, args| {
            event!(Target::Widget(args.widget.id), ScrollParentEvent::WidgetMouseWheel(event.clone()));
        });

        ScrollBuilder {
            widget: widget,
            content_holder: content_holder,
        }
    }
    pub fn add_content<C: BuildWidget>(&mut self, widget: C) -> &mut Self {
        self.content_holder.add_child(widget);
        self
    }
}
widget_builder!(ScrollBuilder, build: |mut builder: ScrollBuilder| -> WidgetBuilder {
    builder.widget.add_child(builder.content_holder);
    builder.widget
});

struct ScrollContainer;
impl LayoutContainer for ScrollContainer {
    fn add_child(&mut self, parent: &Widget, child: &mut WidgetBuilder) {
        event!(Target::Widget(parent.id), ScrollParentEvent::ChildAttached(Some(child.id())));
        let ref parent = parent.layout;
        // only used to set initial position
        layout!(child:
            align_left(parent).strength(WEAK),
            align_top(parent).strength(WEAK),
        );
        child.add_handler(WidgetScrollHandler::new());
    }
    fn remove_child(&mut self, parent: &Widget, _: WidgetId, _: &mut LimnSolver) {
        event!(Target::Widget(parent.id), ScrollParentEvent::ChildAttached(None));
    }
}

enum ScrollParentEvent {
    ChildAttached(Option<WidgetId>),
    WidgetMouseWheel(WidgetMouseWheel),
}
struct ScrollParent {
    scrollable: Option<WidgetId>,
}
impl ScrollParent {
    fn new() -> Self {
        ScrollParent {
            scrollable: None,
        }
    }
}
impl WidgetEventHandler<ScrollParentEvent> for ScrollParent {
    fn handle(&mut self, event: &ScrollParentEvent, args: WidgetEventArgs) {
        match *event {
            ScrollParentEvent::ChildAttached(ref child_id) => {
                if self.scrollable.is_some() {
                    panic!("Scroll parent has more than one child");
                }
                self.scrollable = child_id.clone();
            }
            ScrollParentEvent::WidgetMouseWheel(ref mouse_wheel) => {
                if let Some(scrollable) = self.scrollable {
                    let widget_bounds = args.widget.bounds;
                    let event = WidgetScroll {
                        event: mouse_wheel.0,
                        parent_bounds: widget_bounds,
                    };
                    event!(Target::Widget(scrollable), event);
                }
            }
        }
    }
}

pub struct WidgetScroll {
    event: glutin::MouseScrollDelta,
    parent_bounds: Rect,
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
        let &WidgetScroll { event, parent_bounds } = event;
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
}
