use std::collections::{HashSet, HashMap, VecDeque};
use std::any::{Any, TypeId};
use std::rc::Rc;
use std::cell::RefCell;

use cassowary::strength::*;

use graphics;
use graphics::Context;

use backend::gfx::G2d;
use backend::glyph::GlyphCache;
use backend::window::Window;

use app::App;
use widget::Widget;
use layout::{LayoutManager, LayoutVars};
use layout::constraint::*;
use util::{Point, Rect, Size};
use resources::WidgetId;
use event::Target;

pub struct Ui {
    pub root: Widget,
    widget_map: HashMap<WidgetId, Widget>,
    pub layout: LayoutManager,
    glyph_cache: GlyphCache,
    pub needs_redraw: bool,
    should_close: bool,
    debug_draw_bounds: bool,
    pub window: Rc<RefCell<Window>>,
}

impl Ui {
    pub fn new(mut window: Window) -> Self {
        let mut layout = LayoutManager::new();
        let mut root = Widget::new_named("root");
        layout!(root: top_left(Point::zero()));
        {
            let ref root_vars = root.layout().vars;
            layout.solver.update_solver(|solver| {
                solver.add_edit_variable(root_vars.right, REQUIRED - 1.0).unwrap();
                solver.add_edit_variable(root_vars.bottom, REQUIRED - 1.0).unwrap();
            });
            layout.check_changes();
        }
        root.add_handler_fn(|_: &::layout::LayoutUpdated, _| {
            event!(Target::Ui, ::layout::ResizeWindow);
        });
        Ui {
            widget_map: HashMap::new(),
            root: root,
            layout: layout,
            glyph_cache: GlyphCache::new(&mut window.context.factory, 512, 512),
            needs_redraw: false,
            should_close: false,
            debug_draw_bounds: false,
            window: Rc::new(RefCell::new(window)),
        }
    }
    pub fn get_widget(&mut self, widget_id: WidgetId) -> Option<Widget> {
        self.widget_map.get(&widget_id).map(|widget| widget.clone())
    }
    pub fn get_root(&mut self) -> Widget {
        self.root.clone()
    }

    pub fn widgets_under_cursor(&mut self, point: Point) -> CursorWidgetWalker {
        CursorWidgetWalker::new(point, self.get_root())
    }
    pub fn close(&mut self) {
        self.should_close = true;
    }
    pub fn should_close(&self) -> bool {
        self.should_close
    }
    pub fn set_debug_draw_bounds(&mut self, debug_draw_bounds: bool) {
        self.debug_draw_bounds = debug_draw_bounds;
        self.redraw();
    }
    pub fn resize_window_to_fit(&mut self) {
        let window_dims = self.get_root_dims();
        self.window.borrow_mut().window.set_inner_size(window_dims.width as u32, window_dims.height as u32);
    }
    pub fn get_root_dims(&mut self) -> Size {
        let root = self.get_root();
        let mut dims = root.bounds().size;
        // use min size to prevent window size from being set to 0 (X crashes)
        dims.width = f64::max(100.0, dims.width);
        dims.height = f64::max(100.0, dims.height);
        dims
    }
    pub fn window_resized(&mut self, window_dims: Size) {
        self.window.borrow_mut().window_resized();
        let mut root = self.get_root();
        root.update_layout(|layout| {
            layout.edit_right().set(window_dims.width);
            layout.edit_bottom().set(window_dims.height);
        });
        self.layout.check_changes();
        self.needs_redraw = true;
    }

    pub fn redraw(&mut self) {
        self.needs_redraw = true;
    }
    pub fn draw_if_needed(&mut self) {
        if self.needs_redraw {
            let window = self.window.clone();
            window.borrow_mut().draw_2d(|context, graphics| {
                graphics::clear([0.8, 0.8, 0.8, 1.0], graphics);
                self.draw(context, graphics);
            });
            self.needs_redraw = false;
        }
    }
    pub fn draw(&mut self, context: Context, graphics: &mut G2d) {
        let crop_to = Rect::new(Point::zero(), Size::new(::std::f64::MAX, ::std::f64::MAX));
        let root = self.get_root();
        root.widget_mut().draw(crop_to, &mut self.glyph_cache, context, graphics);
        if self.debug_draw_bounds {
            root.widget_mut().draw_debug(context, graphics);
        }
    }

    pub fn widget_under_cursor(&mut self, point: Point) -> Option<Widget> {
        // first widget found is the deepest, later will need to have z order as ordering
        self.widgets_under_cursor(point).next()
    }
    pub fn bfs(&mut self) -> Bfs {
        Bfs::new(self.get_root())
    }

    fn handle_widget_event(&mut self,
                           widget_ref: Widget,
                           type_id: TypeId,
                           data: &Box<Any>) -> bool
    {
        let handled = widget_ref.trigger_event(type_id, data);
        if widget_ref.has_updated() {
            self.needs_redraw = true;
            widget_ref.set_updated(false);
        }
        handled
    }

    pub fn handle_event(&mut self,
                        address: Target,
                        type_id: TypeId,
                        data: &Box<Any>) {
        match address {
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
                    maybe_widget_ref = widget_ref.widget().parent.as_ref().and_then(|parent| parent.upgrade());
                }
            }
            _ => ()
        }
    }
    fn handle_event_subtree(&mut self, widget_ref: Widget, type_id: TypeId, data: &Box<Any>) {
        self.handle_widget_event(widget_ref.clone(), type_id, data);
        let children = &widget_ref.widget().children;
        for child in children {
            self.handle_event_subtree(child.clone(), type_id, data);
        }
    }
    pub fn debug_widget_positions(&mut self) {
        println!("WIDGET POSITIONS");
        let mut bfs = self.bfs();
        while let Some(widget_ref) = bfs.next() {
            let bounds = widget_ref.bounds();
            let name = widget_ref.debug_name().clone();
            println!("{:?} {:?}", name, bounds);
        }
    }
    pub fn debug_constraints(&mut self) {
        println!("CONSTRAINTS");
        let root = self.get_root();
        root.widget().debug_constraints();
    }
    pub fn debug_variables(&mut self) {
        self.layout.solver.debug_variables();
    }
}

#[derive(Clone)]
pub struct RegisterWidget(pub Widget);
#[derive(Clone)]
pub struct RemoveWidget(pub Widget);

impl App {
    pub fn add_ui_handlers(&mut self) {
        self.add_handler_fn(|event: &RegisterWidget, ui| {
            let event = event.clone();
            let RegisterWidget(mut widget_ref) = event;
            ui.layout.solver.register_widget(widget_ref.id().0, &widget_ref.debug_name(), &mut widget_ref.layout());
            ui.widget_map.insert(widget_ref.id(), widget_ref.clone());
        });
        self.add_handler_fn(|event: &RemoveWidget, ui| {
            let event = event.clone();
            let RemoveWidget(widget_ref) = event;
            ui.layout.solver.remove_widget(widget_ref.id().0);
            ui.layout.check_changes();
            ui.widget_map.remove(&widget_ref.id());
        });
    }
}
pub struct WidgetAttachedEvent;
pub struct WidgetDetachedEvent;
pub struct ChildAttachedEvent(pub WidgetId, pub LayoutVars);



pub struct CursorWidgetWalker {
    point: Point,
    dfs: DfsPostReverse,
}
impl CursorWidgetWalker {
    fn new(point: Point, root: Widget) -> Self {
        CursorWidgetWalker {
            point: point,
            dfs: DfsPostReverse::new(root),
        }
    }
    pub fn next(&mut self) -> Option<Widget> {
        while let Some(widget_ref) = self.dfs.next() {
            let ref widget = widget_ref.widget();
            if widget.is_mouse_over(self.point) {
                return Some(widget_ref.clone());
            }
        }
        None
    }
}

// iterates in reverse of draw order, that is, depth first post order,
// with siblings in reverse of insertion order
pub struct DfsPostReverse {
    stack: Vec<Widget>,
    discovered: HashSet<Widget>,
    finished: HashSet<Widget>,
}

impl DfsPostReverse {
    fn new(root: Widget) -> Self {
        DfsPostReverse {
            stack: vec![root],
            discovered: HashSet::new(),
            finished: HashSet::new(),
        }
    }
    pub fn next(&mut self) -> Option<Widget> {
        while let Some(widget_ref) = self.stack.last().map(|w| w.clone()) {
            if self.discovered.insert(widget_ref.clone()) {
                for child in &widget_ref.widget().children {
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

pub struct Bfs {
    queue: VecDeque<Widget>,
}

impl Bfs {
    fn new(root: Widget) -> Self {
        let mut queue = VecDeque::new();
        queue.push_front(root);
        Bfs { queue: queue }
    }
    pub fn next(&mut self) -> Option<Widget> {
        while let Some(widget_ref) = self.queue.pop_front() {
            for child in &widget_ref.widget().children {
                self.queue.push_back(child.clone());
            }
            return Some(widget_ref);
        }
        None
    }
}
