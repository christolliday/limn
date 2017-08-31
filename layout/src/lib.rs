extern crate cassowary;
extern crate euclid;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate maplit;

use std::collections::HashSet;
use std::ops::Drop;
use std::mem;

use cassowary::{Variable, Constraint};
use cassowary::WeightedRelation::*;
use cassowary::strength::*;

use self::constraint::ConstraintBuilder;

use euclid::{Point2D, Size2D, UnknownUnit};

pub type Length = euclid::Length<f32, UnknownUnit>;
pub type Size = Size2D<f32>;
pub type Point = Point2D<f32>;
pub type Rect = euclid::Rect<f32>;

pub type LayoutId = usize;

#[derive(Clone)]
pub struct LayoutVars {
    pub left: Variable,
    pub top: Variable,
    pub right: Variable,
    pub bottom: Variable,
    pub width: Variable,
    pub height: Variable,
}
impl LayoutVars {
    pub fn new() -> Self {
        LayoutVars {
            left: Variable::new(),
            top: Variable::new(),
            right: Variable::new(),
            bottom: Variable::new(),
            width: Variable::new(),
            height: Variable::new(),
        }
    }
    pub fn array(&self) -> [Variable; 6] {
        [self.left, self.top, self.right, self.bottom, self.width, self.height]
    }
}

#[derive(Debug, Clone, Copy)]
pub enum VarType {
    Left,
    Top,
    Right,
    Bottom,
    Width,
    Height,
}

pub trait LayoutRef {
    fn layout_ref(&self) -> LayoutVars;
}

impl<'a> LayoutRef for &'a mut Layout {
    fn layout_ref(&self) -> LayoutVars {
        self.vars.clone()
    }
}
impl LayoutRef for Layout {
    fn layout_ref(&self) -> LayoutVars {
        self.vars.clone()
    }
}
impl LayoutRef for LayoutVars {
    fn layout_ref(&self) -> LayoutVars {
        self.clone()
    }
}

pub struct Layout {
    pub vars: LayoutVars,
    pub name: Option<String>,
    pub id: LayoutId,
    edit_vars: Vec<EditVariable>,
    constraints: HashSet<Constraint>,
    new_constraints: HashSet<Constraint>,
    removed_constraints: Vec<Constraint>,
}
impl Layout {
    pub fn new(id: LayoutId, name: Option<String>) -> Self {
        let vars = LayoutVars::new();
        let mut new_constraints = HashSet::new();
        new_constraints.insert(vars.right - vars.left| EQ(REQUIRED) | vars.width);
        new_constraints.insert(vars.bottom - vars.top | EQ(REQUIRED) | vars.height);
        // temporarily disabling this, as it tends to cause width/height to snap to 0
        //new_constraints.insert(vars.width | GE(REQUIRED) | 0.0);
        //new_constraints.insert(vars.height | GE(REQUIRED) | 0.0);
        Layout {
            vars: vars,
            name: name,
            id: id,
            edit_vars: Vec::new(),
            constraints: HashSet::new(),
            new_constraints: new_constraints,
            removed_constraints: Vec::new(),
        }
    }
    pub fn layout(&mut self) -> &mut Self {
        self
    }
    pub fn edit_left(&mut self) -> VariableEditable {
        let var = self.vars.left;
        VariableEditable::new(self, var)
    }
    pub fn edit_top(&mut self) -> VariableEditable {
        let var = self.vars.top;
        VariableEditable::new(self, var)
    }
    pub fn edit_right(&mut self) -> VariableEditable {
        let var = self.vars.right;
        VariableEditable::new(self, var)
    }
    pub fn edit_bottom(&mut self) -> VariableEditable {
        let var = self.vars.bottom;
        VariableEditable::new(self, var)
    }
    pub fn edit_width(&mut self) -> VariableEditable {
        let var = self.vars.width;
        VariableEditable::new(self, var)
    }
    pub fn edit_height(&mut self) -> VariableEditable {
        let var = self.vars.height;
        VariableEditable::new(self, var)
    }
    pub fn add<B: ConstraintBuilder>(&mut self, builder: B) {
        let new_constraints = builder.build(self);
        self.new_constraints.extend(new_constraints);
    }
    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.new_constraints.insert(constraint);
    }
    pub fn add_constraints(&mut self, constraints: Vec<Constraint>) {
        self.new_constraints.extend(constraints);
    }
    pub fn remove_constraint(&mut self, constraint: Constraint) {
        if !self.new_constraints.remove(&constraint) {
            self.removed_constraints.push(constraint);
        }
    }
    pub fn get_constraints(&mut self) -> HashSet<Constraint> {
        let new_constraints = mem::replace(&mut self.new_constraints, HashSet::new());
        for constraint in new_constraints.clone() {
            self.constraints.insert(constraint);
        }
        new_constraints
    }
    pub fn get_removed_constraints(&mut self) -> Vec<Constraint> {
        let removed_constraints = mem::replace(&mut self.removed_constraints, Vec::new());
        for ref constraint in &removed_constraints {
            self.constraints.remove(constraint);
        }
        removed_constraints
    }
    pub fn get_edit_vars(&mut self) -> Vec<EditVariable> {
        mem::replace(&mut self.edit_vars, Vec::new())
    }
}

pub struct VariableEditable<'a> {
    pub builder: &'a mut Layout,
    pub var: Variable,
    val: Option<f64>,
    strength: f64,
}
impl<'a> VariableEditable<'a> {
    pub fn new(builder: &'a mut Layout, var: Variable) -> Self {
        VariableEditable {
            builder: builder,
            var: var,
            val: None,
            strength: STRONG,
        }
    }
    pub fn strength(mut self, strength: f64) -> Self {
        self.strength = strength;
        self
    }
    pub fn set(mut self, val: f32) -> Self {
        self.val = Some(val as f64);
        self
    }
}
impl<'a> Drop for VariableEditable<'a> {
    fn drop(&mut self) {
        let edit_var = EditVariable::new(&self);
        self.builder.edit_vars.push(edit_var);
    }
}
#[derive(Debug)]
pub struct EditVariable {
    var: Variable,
    val: Option<f64>,
    strength: f64,
}
impl EditVariable {
    fn new(editable: &VariableEditable) -> Self {
        EditVariable {
            var: editable.var,
            val: editable.val,
            strength: editable.strength,
        }
    }
}

pub fn change_strength(constraints: &Vec<Constraint>, strength: f64) -> Vec<Constraint> {
    let mut new_constraints = Vec::new();
    for cons in constraints {
        let cons = Constraint::new(cons.0.expression.clone(), cons.0.op, strength);
        new_constraints.push(cons);
    }
    new_constraints
}

#[macro_export]
macro_rules! layout {
    ($widget:ident: $($func:expr),*) => {
        layout!($widget: $($func, )*);
    };
    ($widget:ident: $($func:expr,)*) => {
        {
            #[allow(unused_imports)]
            use $crate::constraint::*;
            $(
                $widget.layout().add($func);
            )*
        }
    };
}

lazy_static! {
    pub static ref LAYOUT: LayoutVars = LayoutVars::new();
}

pub mod solver;
pub mod constraint;
pub mod linear_layout;
pub mod grid_layout;

pub use self::solver::LimnSolver;

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use cassowary::strength::*;

    use super::{LimnSolver, LayoutId, Layout, VarType};
    use super::{Size, Point, Rect};

    #[test]
    fn one_widget() {
        let mut layout = TestLayout::new();

        let mut widget = layout.new_widget("widget");
        layout!(widget:
            top_left(Point::new(0.0, 0.0)),
            size(Size::new(200.0, 200.0)),
        );
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
        layout!(widget_o:
            top_left(Point::new(0.0, 0.0)),
            size(Size::new(300.0, 300.0)),
        );
        layout!(widget_tl:
            align_top(&widget_o),
            align_left(&widget_o),
        );
        layout!(widget_tr:
            to_right_of(&widget_tl),
            align_top(&widget_o),
            align_right(&widget_o),
            match_width(&widget_tl),
        );
        layout!(widget_bl:
            below(&widget_tl),
            align_bottom(&widget_o),
            align_left(&widget_o),
            match_width(&widget_tl),
            match_height(&widget_tl),
        );
        layout!(widget_br:
            below(&widget_tr),
            to_right_of(&widget_bl),
            align_bottom(&widget_o),
            align_right(&widget_o),
            match_width(&widget_bl),
            match_height(&widget_tr),
        );
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
        layout!(root_widget:
            top_left(Point::new(0.0, 0.0)),
        );
        root_widget.edit_right().set(100.0).strength(STRONG);
        root_widget.edit_bottom().set(100.0).strength(STRONG);
        layout!(slider:
            align_left(&root_widget).padding(50.0),
        );
        layout!(slider_bar_pre:
            to_left_of(&slider_handle),
        );

        layout!(slider_handle:
            bound_by(&slider),
        );
        layout!(slider_bar_pre:
            bound_by(&slider),
        );
        layout!(slider:
            bound_by(&root_widget),
        );
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
}
