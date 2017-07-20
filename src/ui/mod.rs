pub mod graph;

use backend::gfx::G2d;
use backend::glyph::GlyphCache;
use backend::window::Window;

use std::any::{Any, TypeId};

use cassowary::strength::*;

use graphics;
use graphics::Context;

use widget::{WidgetRef, WidgetBuilder, WidgetBuilderCore};
use layout::{LayoutManager, LayoutVars, LayoutAdded};
use layout::constraint::*;
use util::{self, Point, Rect, Size};
use resources::WidgetId;
use color::*;
use event::Target;

use ui::graph::WidgetGraph;

pub struct Ui {
    pub graph: WidgetGraph,
    pub layout: LayoutManager,
    glyph_cache: GlyphCache,
    needs_redraw: bool,
    should_close: bool,
    debug_draw_bounds: bool,
}

impl Ui {
    pub fn new(window: &mut Window) -> Self {
        Ui {
            graph: WidgetGraph::new(),
            layout: LayoutManager::new(),
            glyph_cache: GlyphCache::new(&mut window.context.factory, 512, 512),
            needs_redraw: false,
            should_close: false,
            debug_draw_bounds: false,
        }
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
    pub fn resize_window_to_fit(&mut self, window: &Window) {
        let window_dims = self.get_root_dims();
        window.window.set_inner_size(window_dims.width as u32, window_dims.height as u32);
    }
    pub fn set_root(&mut self, mut root_widget: WidgetBuilder) {
        root_widget.set_debug_name("root");
        layout!(root_widget: top_left(Point::zero()));
        {
            let ref root_vars = root_widget.layout().vars;
            self.layout.solver.update_solver(|solver| {
                solver.add_edit_variable(root_vars.right, REQUIRED - 1.0).unwrap();
                solver.add_edit_variable(root_vars.bottom, REQUIRED - 1.0).unwrap();
            });
            self.layout.check_changes();
        }
        self.graph.root_id = root_widget.id();
        self.graph.root = Some(self.add_widget(root_widget, None));
    }
    pub fn get_root_dims(&mut self) -> Size {
        let root = self.graph.get_root();
        let mut dims = root.bounds().size;
        // use min size to prevent window size from being set to 0 (X crashes)
        dims.width = f64::max(100.0, dims.width);
        dims.height = f64::max(100.0, dims.height);
        dims
    }
    pub fn window_resized(&mut self, window_dims: Size) {
        let root = self.graph.get_root();
        let mut root = root.0.borrow_mut();
        root.widget.update_layout(|layout| {
            layout.edit_right().set(window_dims.width);
            layout.edit_bottom().set(window_dims.height);
        });
        self.layout.check_changes();
        self.needs_redraw = true;
    }

    pub fn redraw(&mut self) {
        self.needs_redraw = true;
    }
    pub fn draw_if_needed(&mut self, window: &mut Window) {
        if self.needs_redraw {
            window.draw_2d(|context, graphics| {
                graphics::clear([0.8, 0.8, 0.8, 1.0], graphics);
                self.draw(context, graphics);
            });
            self.needs_redraw = false;
        }
    }
    pub fn draw(&mut self, context: Context, graphics: &mut G2d) {
        use std::f64;
        let crop_to = Rect::new(Point::zero(), Size::new(f64::MAX, f64::MAX));
        let root = self.graph.get_root();
        self.draw_node(context, graphics, root, crop_to);
        if self.debug_draw_bounds {
            let root_id = self.graph.root_id;
            let mut bfs = self.graph.bfs(root_id);
            while let Some(widget_ref) = bfs.next() {
                let color = widget_ref.debug_color().unwrap_or(GREEN);
                let bounds = widget_ref.bounds();
                util::draw_rect_outline(bounds, color, context, graphics);
            }
        }
    }
    pub fn draw_node(&mut self,
                     context: Context,
                     graphics: &mut G2d,
                     widget_ref: WidgetRef,
                     crop_to: Rect) {

        let mut widget = widget_ref.widget_container_mut();
        let widget = &mut widget.widget;
        let crop_to = {
            widget.draw(crop_to, &mut self.glyph_cache, context, graphics);
            crop_to.intersection(&widget.bounds)
        };

        if let Some(crop_to) = crop_to {
            for child in &widget.children {
                self.draw_node(context, graphics, child.clone(), crop_to);
            }
        }
    }

    pub fn add_widget(&mut self,
                      builder: WidgetBuilder,
                      parent_id: Option<WidgetId>) -> WidgetRef {
        let (children, widget) = builder.build();
        event!(Target::Ui, LayoutAdded(widget.id()));
        self.layout.check_changes();

        let id = widget.id();
        self.graph.add_widget(widget, parent_id);

        let widget_ref = self.graph.get_widget(id).unwrap();
        if let Some(parent_id) = parent_id {
            if let Some(parent) = self.graph.get_widget(parent_id) {
                let parent_weak = parent.downgrade();
                let parent = &mut *parent.widget_container_mut();
                parent.widget.children.push(widget_ref.clone());
                let widget = &mut (&mut *widget_ref.widget_container_mut()).widget;
                widget.parent = Some(parent_weak);
                if let Some(ref mut container) = parent.container {
                    container.add_child(&mut parent.widget, widget);
                }
                event!(Target::Widget(parent_id), ChildAttachedEvent(id, widget.layout().vars.clone()));
            }
        }
        event!(Target::Widget(id), WidgetAttachedEvent);
        for child in children {
            self.add_widget(child, Some(id));
        }
        self.redraw();
        widget_ref
    }

    pub fn remove_widget(&mut self, widget_id: WidgetId) {
        if let Some(widget_ref) = self.graph.get_widget(widget_id) {
            let widget_container = widget_ref.widget_container();
            if let Some(ref parent_ref) = widget_container.widget.parent {
                if let Some(parent_ref) = parent_ref.upgrade() {
                    let parent = &mut *parent_ref.widget_container_mut();
                    if let Some(ref mut container) = parent.container {
                        container.remove_child(&mut parent.widget, widget_container.widget.id);
                    }
                    parent.widget.remove_child(widget_container.widget.id);
                }
            }
        }
        event!(Target::Widget(widget_id), WidgetDetachedEvent);
        if let Some(_) = self.graph.remove_widget(widget_id) {
            self.layout.solver.remove_widget(widget_id.0);
            self.layout.check_changes();
            self.redraw();
        }
    }

    pub fn widget_under_cursor(&mut self, point: Point) -> Option<WidgetId> {
        // first widget found is the deepest, later will need to have z order as ordering
        self.graph.widgets_under_cursor(point).next()
    }

    fn handle_widget_event(&mut self,
                           widget_ref: WidgetRef,
                           type_id: TypeId,
                           data: &Box<Any + Send>) -> bool
    {
        let mut widget_container = widget_ref.widget_container_mut();
        let handled = widget_container.trigger_event(type_id,
                                                     data,
                                                     &mut self.layout);
        if widget_container.widget.has_updated {
            self.needs_redraw = true;
            widget_container.widget.has_updated = false;
        }
        handled
    }

    pub fn handle_event(&mut self,
                        address: Target,
                        type_id: TypeId,
                        data: &Box<Any + Send>) {
        match address {
            Target::Widget(widget_id) => {
                if let Some(widget_ref) = self.graph.get_widget(widget_id) {
                    self.handle_widget_event(widget_ref, type_id, data);
                }
            }
            Target::SubTree(widget_id) => {
                if let Some(widget_ref) = self.graph.get_widget(widget_id) {
                    self.handle_event_subtree(widget_ref, type_id, data);
                }
            }
            Target::BubbleUp(widget_id) => {
                let mut maybe_widget_ref = self.graph.get_widget(widget_id);
                while let Some(widget_ref) = maybe_widget_ref {
                    if self.handle_widget_event(widget_ref.clone(), type_id, data) {
                        break;
                    }
                    maybe_widget_ref = widget_ref.widget_container().widget.parent.as_ref().and_then(|parent| parent.upgrade());
                }
            }
            _ => ()
        }
    }
    fn handle_event_subtree(&mut self, widget_ref: WidgetRef, type_id: TypeId, data: &Box<Any + Send>) {
        self.handle_widget_event(widget_ref.clone(), type_id, data);
        let children = &widget_ref.widget_container().widget.children;
        for child in children {
            self.handle_event_subtree(child.clone(), type_id, data);
        }
    }
    pub fn debug_widget_positions(&mut self) {
        println!("WIDGET POSITIONS");
        let root_id = self.graph.root_id;
        let mut bfs = self.graph.bfs(root_id);
        while let Some(widget_ref) = bfs.next() {
            let bounds = widget_ref.bounds();
            let name = widget_ref.debug_name().clone();
            println!("{:?} {:?}", name, bounds);
        }
    }
    pub fn debug_constraints(&mut self) {
        self.layout.solver.debug_constraints();
    }
    pub fn debug_variables(&mut self) {
        self.layout.solver.debug_variables();
    }
}
pub struct WidgetAttachedEvent;
pub struct WidgetDetachedEvent;
pub struct ChildAttachedEvent(pub WidgetId, pub LayoutVars);
