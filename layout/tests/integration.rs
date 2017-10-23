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

use layout::{LimnSolver, LayoutId, Layout, VarType, LayoutRef, LayoutVars};
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

    layout.add_root(widget.clone());
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
        align_to_right_of(&widget_tl),
        align_top(&widget_o),
        align_right(&widget_o),
        match_width(&widget_tl),
    ]);
    widget_bl.add(constraints![
        align_below(&widget_tl),
        align_bottom(&widget_o),
        align_left(&widget_o),
        match_width(&widget_tl),
        match_height(&widget_tl),
    ]);
    widget_br.add(constraints![
        align_below(&widget_tr),
        align_to_right_of(&widget_bl),
        align_bottom(&widget_o),
        align_right(&widget_o),
        match_width(&widget_bl),
        match_height(&widget_tr),
    ]);

    layout.add_root(widget_o.clone());
    layout.add_root(widget_tl.clone());
    layout.add_root(widget_tr.clone());
    layout.add_root(widget_bl.clone());
    layout.add_root(widget_br.clone());
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
    let grid_layout = GridLayout::new(&mut grid, 2);
    grid.set_container(grid_layout);

    let widgets = {
        let mut widgets = Vec::new();
        for i in 0..4 {
            let mut widget = layout.new_widget(&format!("widget_{}", i));
            grid.add_child(widget.deref_mut());
            widgets.push(widget);
        }
        widgets
    };

    layout.add_root(grid.clone());
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

    let mut root = layout.new_widget("root");
    let mut slider = layout.new_widget("slider");
    let mut slider_bar_pre = layout.new_widget("slider_bar_pre");
    let mut slider_handle = layout.new_widget("slider_handle");
    root.add(top_left(Point::new(0.0, 0.0)));
    root.edit_right().set(100.0).strength(STRONG);
    root.edit_bottom().set(100.0).strength(STRONG);
    slider.add(constraints![
        align_left(&root).padding(50.0),
        bound_by(&root),
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

    layout.add_root(root);
    layout.update();
}

#[test]
fn linear_layout_fill() {
    use layout::linear_layout::{LinearLayout, LinearLayoutSettings, Orientation, ItemAlignment};

    let mut layout = TestLayout::new();

    let mut root = layout.new_widget("root");
    let mut item_1 = layout.new_widget("item_1");
    let mut item_2 = layout.new_widget("item_2");
    let mut item_3 = layout.new_widget("item_3");

    root.add(constraints![
        top_left(Point::new(0.0, 0.0)),
        size(Size::new(300.0, 10.0))
    ]);
    let mut settings = LinearLayoutSettings::new(Orientation::Horizontal);
    settings.fill_equal = true;
    settings.item_align = ItemAlignment::Fill;
    let linear_layout = LinearLayout::new(&mut *root, settings);
    root.set_container(linear_layout);

    root.add_child(item_1.deref_mut());
    root.add_child(item_2.deref_mut());
    root.add_child(item_3.deref_mut());

    layout.add_root(root.clone());
    layout.update();
    assert!(layout.layout_rects == hashmap!{
        root.id => Rect::new(Point::new(0.0, 0.0), Size::new(300.0, 10.0)),
        item_1.id => Rect::new(Point::new(0.0, 0.0), Size::new(100.0, 10.0)),
        item_2.id => Rect::new(Point::new(100.0, 0.0), Size::new(100.0, 10.0)),
        item_3.id => Rect::new(Point::new(200.0, 0.0), Size::new(100.0, 10.0)),
    });
}

#[test]
fn linear_layout_end_padding() {
    use layout::linear_layout::{LinearLayout, LinearLayoutSettings, Orientation, ItemAlignment, Spacing};

    let mut layout = TestLayout::new();

    let mut root = layout.new_widget("root");
    let mut item_1 = layout.new_widget("item_1");
    let mut item_2 = layout.new_widget("item_2");
    let mut item_3 = layout.new_widget("item_3");

    root.add(constraints![
        top_left(Point::new(0.0, 0.0)),
        size(Size::new(100.0, 10.0))
    ]);
    item_1.add(width(20.0));
    item_2.add(width(20.0));
    item_3.add(width(20.0));
    let mut settings = LinearLayoutSettings::new(Orientation::Horizontal);
    settings.padding = 10.0;
    settings.item_align = ItemAlignment::Fill;
    settings.spacing = Spacing::End;
    let linear_layout = LinearLayout::new(&mut *root, settings);
    root.set_container(linear_layout);

    root.add_child(item_1.deref_mut());
    root.add_child(item_2.deref_mut());
    root.add_child(item_3.deref_mut());

    layout.add_root(root.clone());
    layout.update();
    assert!(layout.layout_rects == hashmap!{
        root.id => Rect::new(Point::new(0.0, 0.0), Size::new(100.0, 10.0)),
        item_1.id => Rect::new(Point::new(0.0, 0.0), Size::new(20.0, 10.0)),
        item_2.id => Rect::new(Point::new(30.0, 0.0), Size::new(20.0, 10.0)),
        item_3.id => Rect::new(Point::new(60.0, 0.0), Size::new(20.0, 10.0)),
    });
}

#[test]
fn linear_layout_start() {
    use layout::linear_layout::{LinearLayout, LinearLayoutSettings, Orientation, ItemAlignment, Spacing};

    let mut layout = TestLayout::new();

    let mut root = layout.new_widget("root");
    let mut item_1 = layout.new_widget("item_1");
    let mut item_2 = layout.new_widget("item_2");
    let mut item_3 = layout.new_widget("item_3");

    root.add(constraints![
        top_left(Point::new(0.0, 0.0)),
        size(Size::new(100.0, 10.0))
    ]);
    item_1.add(width(20.0));
    item_2.add(width(20.0));
    item_3.add(width(20.0));
    let mut settings = LinearLayoutSettings::new(Orientation::Horizontal);
    settings.item_align = ItemAlignment::Fill;
    settings.spacing = Spacing::Start;
    let linear_layout = LinearLayout::new(&mut *root, settings);
    root.set_container(linear_layout);

    root.add_child(item_1.deref_mut());
    root.add_child(item_2.deref_mut());
    root.add_child(item_3.deref_mut());

    layout.add_root(root.clone());
    layout.update();
    assert!(layout.layout_rects == hashmap!{
        root.id => Rect::new(Point::new(0.0, 0.0), Size::new(100.0, 10.0)),
        item_1.id => Rect::new(Point::new(40.0, 0.0), Size::new(20.0, 10.0)),
        item_2.id => Rect::new(Point::new(60.0, 0.0), Size::new(20.0, 10.0)),
        item_3.id => Rect::new(Point::new(80.0, 0.0), Size::new(20.0, 10.0)),
    });
}

#[test]
fn linear_layout_spacing_around() {
    use layout::linear_layout::{LinearLayout, LinearLayoutSettings, Orientation, ItemAlignment, Spacing};

    let mut layout = TestLayout::new();

    let mut root = layout.new_widget("root");
    let mut item_1 = layout.new_widget("item_1");
    let mut item_2 = layout.new_widget("item_2");
    let mut item_3 = layout.new_widget("item_3");

    root.add(constraints![
        top_left(Point::new(0.0, 0.0)),
        size(Size::new(100.0, 10.0))
    ]);
    item_1.add(width(20.0));
    item_2.add(width(20.0));
    item_3.add(width(20.0));
    let mut settings = LinearLayoutSettings::new(Orientation::Horizontal);
    settings.item_align = ItemAlignment::Fill;
    settings.spacing = Spacing::Around;
    let linear_layout = LinearLayout::new(&mut *root, settings);
    root.set_container(linear_layout);

    root.add_child(item_1.deref_mut());
    root.add_child(item_2.deref_mut());
    root.add_child(item_3.deref_mut());

    layout.add_root(root.clone());
    layout.update();
    assert!(layout.layout_rects == hashmap!{
        root.id => Rect::new(Point::new(0.0, 0.0), Size::new(100.0, 10.0)),
        item_1.id => Rect::new(Point::new(10.0, 0.0), Size::new(20.0, 10.0)),
        item_2.id => Rect::new(Point::new(40.0, 0.0), Size::new(20.0, 10.0)),
        item_3.id => Rect::new(Point::new(70.0, 0.0), Size::new(20.0, 10.0)),
    });
}

#[test]
fn linear_layout_spacing_between() {
    use layout::linear_layout::{LinearLayout, LinearLayoutSettings, Orientation, ItemAlignment, Spacing};

    let mut layout = TestLayout::new();

    let mut root = layout.new_widget("root");
    let mut item_1 = layout.new_widget("item_1");
    let mut item_2 = layout.new_widget("item_2");
    let mut item_3 = layout.new_widget("item_3");

    root.add(constraints![
        top_left(Point::new(0.0, 0.0)),
        size(Size::new(100.0, 10.0))
    ]);
    item_1.add(width(20.0));
    item_2.add(width(20.0));
    item_3.add(width(20.0));
    let mut settings = LinearLayoutSettings::new(Orientation::Horizontal);
    settings.item_align = ItemAlignment::Fill;
    settings.spacing = Spacing::Between;
    let linear_layout = LinearLayout::new(&mut *root, settings);
    root.set_container(linear_layout);

    root.add_child(item_1.deref_mut());
    root.add_child(item_2.deref_mut());
    root.add_child(item_3.deref_mut());

    layout.add_root(root.clone());
    layout.update();
    assert!(layout.layout_rects == hashmap!{
        root.id => Rect::new(Point::new(0.0, 0.0), Size::new(100.0, 10.0)),
        item_1.id => Rect::new(Point::new(0.0, 0.0), Size::new(20.0, 10.0)),
        item_2.id => Rect::new(Point::new(40.0, 0.0), Size::new(20.0, 10.0)),
        item_3.id => Rect::new(Point::new(80.0, 0.0), Size::new(20.0, 10.0)),
    });
}

#[test]
fn linear_layout_remove() {
    use layout::linear_layout::{LinearLayout, LinearLayoutSettings, Orientation, ItemAlignment, Spacing};

    let mut layout = TestLayout::new();

    let mut root = layout.new_widget("root");
    let mut item_1 = layout.new_widget("item_1");
    let mut item_2 = layout.new_widget("item_2");
    let mut item_3 = layout.new_widget("item_3");

    root.add(constraints![
        top_left(Point::new(0.0, 0.0)),
        size(Size::new(100.0, 10.0))
    ]);
    item_1.add(width(20.0));
    item_2.add(width(20.0));
    item_3.add(width(20.0));
    let mut settings = LinearLayoutSettings::new(Orientation::Horizontal);
    settings.fill_equal = true;
    settings.item_align = ItemAlignment::Fill;
    settings.spacing = Spacing::End;
    settings.padding = 10.0;
    let linear_layout = LinearLayout::new(&mut *root, settings);
    root.set_container(linear_layout);

    root.add_child(&mut *item_1);
    root.add_child(&mut *item_2);
    root.add_child(&mut *item_3);

    root.remove_child(&mut *item_2);

    layout.add_root(root.clone());
    layout.update();
    assert!(layout.match_layouts(hashmap!{
        root.id => Rect::new(Point::new(0.0, 0.0), Size::new(100.0, 10.0)),
        item_1.id => Rect::new(Point::new(0.0, 0.0), Size::new(20.0, 10.0)),
        item_3.id => Rect::new(Point::new(30.0, 0.0), Size::new(20.0, 10.0)),
    }));
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
    layout_rects: HashMap<LayoutId, Rect>,
    layouts: HashMap<LayoutId, SharedLayout>,
    roots: Vec<SharedLayout>,
}
impl TestLayout {
    fn new() -> Self {
        let mut solver = LimnSolver::new();
        solver.strict = true;
        TestLayout {
            id_gen: IdGen::new(),
            solver: solver,
            layout_rects: HashMap::new(),
            layouts: HashMap::new(),
            roots: Vec::new(),
        }
    }
    fn new_widget(&mut self, name: &str) -> SharedLayout {
        let id = self.id_gen.next();
        let layout = Layout::new(id, Some(name.to_owned()));
        let layout = SharedLayout::new(layout);
        self.layouts.insert(id, layout.clone());
        layout
    }
    fn add_root(&mut self, layout: SharedLayout) {
        self.roots.push(layout);
    }
    fn update_layout(&mut self, mut layout: SharedLayout) {
        self.solver.update_layout(layout.deref_mut());
        for child in layout.get_children() {
            let layout = self.layouts[child].clone();
            self.update_layout(layout);
        }
    }
    fn update(&mut self) {
        for layout in self.roots.clone() {
            self.update_layout(layout);
        }
        for (id, var, value) in self.solver.fetch_changes() {
            let rect = self.layout_rects.entry(id).or_insert(Rect::zero());
            match var {
                VarType::Left => rect.origin.x = value as f32,
                VarType::Top => rect.origin.y = value as f32,
                VarType::Width => rect.size.width = value as f32,
                VarType::Height => rect.size.height = value as f32,
                _ => (),
            }
        }
    }
    fn match_layouts(&self, layouts: HashMap<LayoutId, Rect>) -> bool {
        for (match_layout_id, match_layout_rect) in layouts {
            let layout_rect = self.layout_rects[&match_layout_id];
            if layout_rect != match_layout_rect {
                self.solver.debug_layouts();
                return false;
            }
        }
        true
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct IdGen {
    id: usize,
}

impl Default for IdGen {
    fn default() -> Self {
        IdGen {
            id: 0,
        }
    }
}
impl IdGen {

    fn new() -> Self {
        Self::default()
    }

    fn next(&mut self) -> LayoutId {
        let next = self.id;
        self.id += 1;
        next
    }
}
