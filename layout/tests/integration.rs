extern crate cassowary;
#[macro_use]
extern crate limn_layout as layout;
#[macro_use]
extern crate maplit;

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use cassowary::strength::*;

use layout::{LimnSolver, LayoutId, Layout, VarType, LayoutRef, LayoutVars, LayoutContainer};
use layout::{Size, Point, Rect};
use layout::constraint::*;

#[test]
fn one_widget() {
    let mut layout = TestLayout::new();

    let mut widget = layout.new_widget("widget");
    widget.add(constraints![
        top_left(Point::new(0.0, 0.0)),
        size(Size::new(200.0, 200.0))
    ]);

    layout.update();
    assert!(layout.layout_rects == hashmap!{
        widget.id => Rect::new(Point::new(0.0, 0.0), Size::new(200.0, 200.0)),
    });
}

#[test]
fn grid() {
    let mut layout = TestLayout::new();

    let mut widget_o = layout.new_widget("widget_o");
    let mut widget_tl = layout.new_widget("widget_tl");
    let mut widget_bl = layout.new_widget("widget_bl");
    let mut widget_tr = layout.new_widget("widget_tr");
    let mut widget_br = layout.new_widget("widget_br");
    widget_o.add(constraints![
        top_left(Point::new(0.0, 0.0)),
        size(Size::new(300.0, 300.0)),
    ]);
    widget_tl.add(constraints![
        align_top(&widget_o),
        align_left(&widget_o),
    ]);
    widget_tr.add(constraints![
        to_right_of(&widget_tl),
        align_top(&widget_o),
        align_right(&widget_o),
        match_width(&widget_tl),
    ]);
    widget_bl.add(constraints![
        below(&widget_tl),
        align_bottom(&widget_o),
        align_left(&widget_o),
        match_width(&widget_tl),
        match_height(&widget_tl),
    ]);
    widget_br.add(constraints![
        below(&widget_tr),
        to_right_of(&widget_bl),
        align_bottom(&widget_o),
        align_right(&widget_o),
        match_width(&widget_bl),
        match_height(&widget_tr),
    ]);

    layout.update();
    assert!(layout.layout_rects == hashmap!{
        widget_o.id => Rect::new(Point::new(0.0, 0.0), Size::new(300.0, 300.0)),
        widget_tl.id => Rect::new(Point::new(0.0, 0.0), Size::new(150.0, 150.0)),
        widget_tr.id => Rect::new(Point::new(150.0, 0.0), Size::new(150.0, 150.0)),
        widget_bl.id => Rect::new(Point::new(0.0, 150.0), Size::new(150.0, 150.0)),
        widget_br.id => Rect::new(Point::new(150.0, 150.0), Size::new(150.0, 150.0)),
    });
}
#[test]
fn grid_layout() {
    use layout::grid_layout::GridLayout;
    let mut layout = TestLayout::new();

    let mut grid = layout.new_widget("grid");
    grid.add(constraints![
        top_left(Point::new(0.0, 0.0)),
        size(Size::new(200.0, 200.0)),
    ]);
    let mut widgets = {
        let mut widgets = Vec::new();
        for i in 0..4 {
            widgets.push(layout.new_widget(&format!("widget_{}", i)));
        }
        widgets
    };

    let mut grid_layout = GridLayout::new(&mut grid, 2);
    for ref mut widget in &mut widgets {
        grid_layout.add_child(&mut grid, widget);
    }

    layout.update();

    assert!(layout.layout_rects == hashmap!{
        grid.id => Rect::new(Point::new(0.0, 0.0), Size::new(200.0, 200.0)),
        widgets[0].id => Rect::new(Point::new(0.0, 0.0), Size::new(100.0, 100.0)),
        widgets[1].id => Rect::new(Point::new(100.0, 0.0), Size::new(100.0, 100.0)),
        widgets[2].id => Rect::new(Point::new(0.0, 100.0), Size::new(100.0, 100.0)),
        widgets[3].id => Rect::new(Point::new(100.0, 100.0), Size::new(100.0, 100.0)),
    });
}

#[test]
fn edit_var() {
    let mut layout = TestLayout::new();

    let mut root_widget = layout.new_widget("root");
    let mut slider = layout.new_widget("slider");
    let mut slider_bar_pre = layout.new_widget("slider_bar_pre");
    let mut slider_handle = layout.new_widget("slider_handle");
    root_widget.add(top_left(Point::new(0.0, 0.0)));
    root_widget.edit_right().set(100.0).strength(STRONG);
    root_widget.edit_bottom().set(100.0).strength(STRONG);
    slider.add(constraints![
        align_left(&root_widget).padding(50.0),
        bound_by(&root_widget),
    ]);
    slider_bar_pre.add(constraints![
        to_left_of(&slider_handle),
        bound_by(&slider),
    ]);
    slider_handle.add(bound_by(&slider));

    let slider_handle_left = slider_handle.layout_ref().left;

    layout.update();

    layout.solver.solver.add_edit_variable(slider_handle_left, STRONG).unwrap();
    layout.solver.solver.suggest_value(slider_handle_left, 50.0).unwrap();

    layout.update();
}

#[derive(Clone)]
struct SharedLayout(Rc<RefCell<Layout>>);
impl SharedLayout {
    fn new(layout: Layout) -> Self {
        SharedLayout(Rc::new(RefCell::new(layout)))
    }
}
impl LayoutRef for SharedLayout {
    fn layout_ref(&self) -> LayoutVars {
        self.0.borrow().vars.clone()
    }
}
impl <'a> Deref for SharedLayout {
    type Target = Layout;
    #[inline]
    fn deref(&self) -> &Layout {
        unsafe {self.0.as_ptr().as_ref().unwrap()}
    }
}
impl <'a> DerefMut for SharedLayout
{   #[inline]
    fn deref_mut(&mut self) -> &mut Layout {
        unsafe {self.0.as_ptr().as_mut().unwrap()}
    }
}

// code below is used to create a test harness for creating layouts outside of the widget graph
struct TestLayout {
    id_gen: IdGen,
    solver: LimnSolver,
    widget_names: HashMap<LayoutId, String>,
    layout_rects: HashMap<LayoutId, Rect>,
    layouts: HashMap<LayoutId, SharedLayout>,
}
impl TestLayout {
    fn new() -> Self {
        TestLayout {
            id_gen: IdGen::new(),
            solver: LimnSolver::new(),
            widget_names: HashMap::new(),
            layout_rects: HashMap::new(),
            layouts: HashMap::new(),
        }
    }
    fn new_widget(&mut self, name: &str) -> SharedLayout {
        let id = self.id_gen.next();
        let layout = Layout::new(id, Some(name.to_owned()));
        self.widget_names.insert(id, name.to_owned());
        let layout = SharedLayout::new(layout);
        self.layouts.insert(id, layout.clone());
        layout
    }
    fn update(&mut self) {
        for mut layout in self.layouts.clone() {
            let layout = layout.1.deref_mut();
            self.solver.update_layout(layout);
        }
        for (id, var, value) in self.solver.fetch_changes() {
            let rect = self.layout_rects.entry(id).or_insert(Rect::zero());
            let name = &self.widget_names[&id];
            println!("{}.{:?} = {}", name, var, value);
            match var {
                VarType::Left => rect.origin.x = value as f32,
                VarType::Top => rect.origin.y = value as f32,
                VarType::Width => rect.size.width = value as f32,
                VarType::Height => rect.size.height = value as f32,
                _ => (),
            }
        }
    }
}
struct IdGen {
    id: usize,
}
impl IdGen {
    fn new() -> Self {
        IdGen {
            id: 0,
        }
    }
    fn next(&mut self) -> LayoutId {
        let next = self.id;
        self.id += 1;
        next
    }
}
