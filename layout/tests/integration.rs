extern crate cassowary;
#[macro_use]
extern crate limn_layout as layout;
#[macro_use]
extern crate maplit;

use std::collections::HashMap;

use cassowary::strength::*;

use layout::{LimnSolver, LayoutId, Layout, VarType};
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
    layout.solver.update_layout(&mut widget);

    layout.update();
    assert!(layout.layout == hashmap!{
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
    layout.solver.update_layout(&mut widget_o);
    layout.solver.update_layout(&mut widget_tl);
    layout.solver.update_layout(&mut widget_tr);
    layout.solver.update_layout(&mut widget_bl);
    layout.solver.update_layout(&mut widget_br);

    layout.update();
    assert!(layout.layout == hashmap!{
        widget_o.id => Rect::new(Point::new(0.0, 0.0), Size::new(300.0, 300.0)),
        widget_tl.id => Rect::new(Point::new(0.0, 0.0), Size::new(150.0, 150.0)),
        widget_tr.id => Rect::new(Point::new(150.0, 0.0), Size::new(150.0, 150.0)),
        widget_bl.id => Rect::new(Point::new(0.0, 150.0), Size::new(150.0, 150.0)),
        widget_br.id => Rect::new(Point::new(150.0, 150.0), Size::new(150.0, 150.0)),
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

    let slider_handle_left = slider_handle.layout().vars.left;

    layout.solver.update_layout(&mut root_widget);
    layout.update();

    layout.solver.update_layout(&mut slider);
    layout.solver.update_layout(&mut slider_bar_pre);
    layout.solver.update_layout(&mut slider_handle);

    layout.solver.solver.add_edit_variable(slider_handle_left, STRONG).unwrap();
    layout.solver.solver.suggest_value(slider_handle_left, 50.0).unwrap();

    layout.update();
}

// code below is used to create a test harness for creating layouts outside of the widget graph
struct TestLayout {
    id_gen: IdGen,
    solver: LimnSolver,
    widget_names: HashMap<LayoutId, String>,
    layout: HashMap<LayoutId, Rect>,
}
impl TestLayout {
    fn new() -> Self {
        TestLayout {
            id_gen: IdGen::new(),
            solver: LimnSolver::new(),
            widget_names: HashMap::new(),
            layout: HashMap::new(),
        }
    }
    fn new_widget(&mut self, name: &str) -> Layout {
        let id = self.id_gen.next();
        let mut layout = Layout::new(id, Some(name.to_owned()));
        self.widget_names.insert(id, name.to_owned());
        self.solver.register_widget(&mut layout);
        layout
    }
    fn update(&mut self) {
        for (widget_id, var, value) in self.solver.fetch_changes() {
            let rect = self.layout.entry(widget_id).or_insert(Rect::zero());
            let name = &self.widget_names[&widget_id];
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
