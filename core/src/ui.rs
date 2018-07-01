//! Contains `Ui`, which contains application global state and is accessible to every event handler.

use std::collections::{HashSet, HashMap, VecDeque};
use std::any::{Any, TypeId};
use std::rc::Rc;
use std::cell::RefCell;

use cassowary::Constraint;
use cassowary::strength::*;

use glutin;

use window::Window;
use app::App;
use widget::Widget;
use layout::{LimnSolver, LayoutChanged, LayoutVars, ExactFrame};
use layout::constraint::*;
use geometry::{Point, Rect, Size};
use resources::WidgetId;
use event::{Target, EventArgs};
use render::WebRenderContext;

/// If true, the constraint that matches the root layout size to the window size
/// is required. This can be useful for debugging but can result in panics from resizing the window.
const WINDOW_CONSTRAINT_REQUIRED: bool = false;

/// The core of a limn application, holds the root of the widget tree and other application global state.
/// `Ui` is accessible to every event handler, so features helper methods that can be accessed at any time.
pub struct Ui {
    pub(crate) root: Widget,
    widget_map: HashMap<WidgetId, Widget>,
    pub(crate) solver: LimnSolver,
    pub(crate) render: WebRenderContext,
    needs_redraw: bool,
    should_close: bool,
    debug_draw_bounds: bool,
    pub window: Rc<RefCell<Window>>,
    window_constraints: Vec<Constraint>,
}

impl Ui {
    pub(super) fn new(mut window: Window, events_loop: &glutin::EventsLoop) -> Self {
        let mut root = Widget::new("window");
        root.layout().set_container(ExactFrame);
        root.layout().add(top_left(Point::zero()));
        // x will crash if window size set to (0, 0)
        root.layout().add(min_size(Size::new(1.0, 1.0)));
        let render = WebRenderContext::new(&mut window, events_loop);
        Ui {
            widget_map: HashMap::new(),
            root: root.into(),
            solver: LimnSolver::new(),
            render: render,
            needs_redraw: true,
            should_close: false,
            debug_draw_bounds: false,
            window: Rc::new(RefCell::new(window)),
            window_constraints: Vec::new(),
        }
    }

    pub fn get_widget(&self, widget_id: WidgetId) -> Option<Widget> {
        self.widget_map.get(&widget_id).cloned()
    }

    pub fn get_root(&self) -> Widget {
        self.root.clone()
    }

    pub fn event<T: 'static>(&self, data: T) {
        self.get_root().event(data);
    }

    pub fn close(&mut self) {
        self.should_close = true;
    }

    pub(super) fn should_close(&self) -> bool {
        self.should_close
    }

    pub(super) fn resize_window_to_fit(&mut self) {
        let window_dims = self.root.bounds().size;
        self.window.borrow_mut().resize(window_dims.width as u32, window_dims.height as u32);
    }

    pub(super) fn window_resized(&mut self, window_dims: Size) {
        let window = self.window.borrow_mut();
        let window_size = window.size_px();
        self.render.window_resized(window_size);
        let mut root = self.get_root();

        if WINDOW_CONSTRAINT_REQUIRED {
            let window_constraints = root.layout().create_constraint(size(window_dims));
            {
                let window_constraints = window_constraints.clone();
                let mut layout = root.layout();
                for constraint in self.window_constraints.drain(..) {
                    layout.remove_constraint(constraint);
                }
                layout.add(window_constraints);
            }
            self.window_constraints = window_constraints;
        } else {
            let mut layout = root.layout();
            layout.edit_right().set(window_dims.width).strength(REQUIRED - 1.0);
            layout.edit_bottom().set(window_dims.height).strength(REQUIRED - 1.0);
        }
        self.needs_redraw = true;
    }

    pub fn check_layout_changes(&mut self) {

        let changes = self.solver.fetch_changes();
        debug!("layout has {} changes", changes.len());
        if !changes.is_empty() {
            self.event(LayoutChanged(changes));
        }
    }

    pub fn redraw(&mut self) {
        self.needs_redraw = true;
    }

    pub fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    pub(super) fn draw_if_needed(&mut self) {
        if self.needs_redraw {
            self.draw();
            self.needs_redraw = false;
        }
    }

    fn draw(&mut self) {
        let window_size = self.window.borrow_mut().size_dp();
        let (builder, resources) = {
            let mut renderer = self.render.render_builder(window_size);
            let crop_to = Rect::new(Point::zero(), Size::new(::std::f32::MAX, ::std::f32::MAX));
            self.root.draw(crop_to, &mut renderer, self.debug_draw_bounds);
            (renderer.builder, renderer.resources)
        };
        self.render.set_display_list(builder, resources, window_size);
        self.render.generate_frame();
    }

    // Call after drawing
    pub(super) fn update(&mut self) {
        self.render.update(self.window.borrow_mut().size_px());
        let window = self.window.borrow_mut();
        window.swap_buffers();
    }

    pub fn widgets_bfs(&self) -> WidgetsBfs {
        WidgetsBfs::new(self.get_root())
    }

    pub fn widgets_under_cursor(&mut self, point: Point) -> WidgetsUnderCursor {
        WidgetsUnderCursor::new(point, self.get_root())
    }

    /// Find the first widget under the cursor, ie. the last to be drawn that is under the cursor
    pub fn widget_under_cursor(&mut self, point: Point) -> Option<Widget> {
        self.widgets_under_cursor(point).next()
    }

    fn handle_widget_event(&mut self, widget_ref: Widget, type_id: TypeId, data: &Any) -> bool {
        let handled = widget_ref.trigger_event(self, type_id, data);
        if widget_ref.has_updated() {
            self.needs_redraw = true;
            widget_ref.set_updated(false);
        }
        handled
    }

    pub(super) fn handle_event(&mut self, address: Target, type_id: TypeId, data: &Any) {
        match address {
            Target::Root => {
                let root = self.get_root();
                self.handle_widget_event(root, type_id, data);
            }
            Target::Widget(widget_ref) => {
                self.handle_widget_event(widget_ref, type_id, data);
            }
            Target::SubTree(widget_ref) => {
                self.handle_event_subtree(widget_ref, type_id, data);
            }
            Target::BubbleUp(widget_ref) => {
                let mut maybe_widget_ref = Some(widget_ref);
                while let Some(widget_ref) = maybe_widget_ref {
                    if self.handle_widget_event(widget_ref.clone(), type_id, data) {
                        break;
                    }
                    maybe_widget_ref = widget_ref.parent();
                }
            }
        }
    }

    fn handle_event_subtree(&mut self, widget_ref: Widget, type_id: TypeId, data: &Any) {
        self.handle_widget_event(widget_ref.clone(), type_id, data);
        let children = &widget_ref.children();
        for child in children {
            self.handle_event_subtree(child.clone(), type_id, data);
        }
    }

    pub fn set_debug_draw_bounds(&mut self, debug_draw_bounds: bool) {
        self.debug_draw_bounds = debug_draw_bounds;
        self.redraw();
    }

    pub fn debug_widget_positions(&self) {
        println!("WIDGET POSITIONS");
        for widget_ref in self.widgets_bfs() {
            let bounds = widget_ref.bounds();
            let name = widget_ref.name();
            println!("{:?} {:?}", name, bounds);
        }
    }

    pub fn print_widgets(&self) {
        for widget_ref in self.widgets_bfs() {
            let draw_state = &widget_ref.widget().draw_state;
            println!("{:?}", draw_state);
        }
    }
}

#[derive(Clone)]
pub struct RegisterWidget(pub Widget);
#[derive(Clone)]
pub struct RemoveWidget(pub Widget);

impl App {
    pub fn add_ui_handlers(&mut self) {
        self.add_handler(|event: &RegisterWidget, args: EventArgs| {
            let event = event.clone();
            let RegisterWidget(widget_ref) = event;
            args.ui.widget_map.insert(widget_ref.id(), widget_ref.clone());
        });
        self.add_handler(|event: &RemoveWidget, args: EventArgs| {
            let event = event.clone();
            let RemoveWidget(widget_ref) = event;
            args.ui.solver.remove_layout(widget_ref.id().0);
            args.ui.check_layout_changes();
            args.ui.widget_map.remove(&widget_ref.id());
        });
    }
}

#[derive(Debug, Copy, Clone)]
pub struct WidgetAttachedEvent;
#[derive(Debug, Copy, Clone)]
pub struct WidgetDetachedEvent;
#[derive(Debug, Copy, Clone)]
pub struct ChildAttachedEvent(pub WidgetId, pub LayoutVars);

pub enum ChildrenUpdatedEvent {
    Added(Widget),
    Removed(Widget),
}


pub struct WidgetsUnderCursor {
    point: Point,
    dfs: WidgetsDfsPostReverse,
}
impl WidgetsUnderCursor {
    fn new(point: Point, root: Widget) -> Self {
        WidgetsUnderCursor {
            point: point,
            dfs: WidgetsDfsPostReverse::new(root),
        }
    }
}

impl Iterator for WidgetsUnderCursor {
    type Item = Widget;
    fn next(&mut self) -> Option<Widget> {
        for widget_ref in self.dfs.by_ref() {
            if widget_ref.is_under_cursor(self.point) {
                return Some(widget_ref.clone());
            }
        }
        None
    }
}

// Iterates in reverse of draw order, that is, depth first post order,
// with siblings in reverse of insertion order
struct WidgetsDfsPostReverse {
    stack: Vec<Widget>,
    discovered: HashSet<Widget>,
    finished: HashSet<Widget>,
}

impl WidgetsDfsPostReverse {
    fn new(root: Widget) -> Self {
        WidgetsDfsPostReverse {
            stack: vec![root],
            discovered: HashSet::new(),
            finished: HashSet::new(),
        }
    }
}

impl Iterator for WidgetsDfsPostReverse {
    type Item = Widget;
    fn next(&mut self) -> Option<Widget> {
        while let Some(widget_ref) = self.stack.last().cloned() {
            if self.discovered.insert(widget_ref.clone()) {
                for child in &widget_ref.children() {
                    self.stack.push(child.clone());
                }
            } else {
                self.stack.pop();
                if self.finished.insert(widget_ref.clone()) {
                    return Some(widget_ref.clone());
                }
            }
        }
        None
    }
}

pub struct WidgetsBfs {
    queue: VecDeque<Widget>,
}

impl WidgetsBfs {
    fn new(root: Widget) -> Self {
        let mut queue = VecDeque::new();
        queue.push_front(root);
        WidgetsBfs { queue: queue }
    }
}

impl Iterator for WidgetsBfs {
    type Item = Widget;
    fn next(&mut self) -> Option<Widget> {
        if let Some(widget_ref) = self.queue.pop_front() {
            for child in &widget_ref.children() {
                self.queue.push_back(child.clone());
            }
            Some(widget_ref)
        } else {
            None
        }
    }
}
