use std::ops::Drop;

use cassowary::{Variable, Constraint};
use cassowary::WeightedRelation::*;
use cassowary::strength::*;

use util::Rect;
use layout::constraint::ConstraintBuilder;

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
    pub fn update_bounds(&self, var: Variable, value: f64, rect: &mut Rect) {
        if var == self.left {
            rect.origin.x = value;
        } else if var == self.top {
            rect.origin.y = value;
        } else if var == self.width {
            rect.size.width = value;
        } else if var == self.height {
            rect.size.height = value;
        } else {
            panic!();
        }
    }
}

pub trait LayoutRef {
    fn layout_ref(&self) -> &LayoutVars;
}

impl<'a> LayoutRef for &'a mut LayoutBuilder {
    fn layout_ref(&self) -> &LayoutVars {
        &self.vars
    }
}
impl LayoutRef for LayoutBuilder {
    fn layout_ref(&self) -> &LayoutVars {
        &self.vars
    }
}
impl LayoutRef for LayoutVars {
    fn layout_ref(&self) -> &LayoutVars {
        self
    }
}

pub struct LayoutBuilder {
    pub vars: LayoutVars,
    pub edit_vars: Vec<EditVariable>,
    pub constraints: Vec<Constraint>,
}
impl LayoutBuilder {
    pub fn new() -> Self {
        let vars = LayoutVars::new();
        let mut constraints = Vec::new();
        constraints.push(vars.right - vars.left| EQ(REQUIRED) | vars.width);
        constraints.push(vars.bottom - vars.top | EQ(REQUIRED) | vars.height);
        // temporarily disabling this, as it tends to cause width/height to snap to 0
        //constraints.push(vars.width | GE(REQUIRED) | 0.0);
        //constraints.push(vars.height | GE(REQUIRED) | 0.0);
        LayoutBuilder {
            vars: vars,
            edit_vars: Vec::new(),
            constraints: constraints,
        }
    }
    pub fn from(vars: LayoutVars) -> Self {
        LayoutBuilder {
            vars: vars,
            edit_vars: Vec::new(),
            constraints: Vec::new(),
        }
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
        let constraints = builder.build(self);
        self.constraints.extend(constraints);
    }
}

pub struct VariableEditable<'a> {
    pub builder: &'a mut LayoutBuilder,
    pub var: Variable,
    val: f64,
    strength: f64,
}
impl<'a> VariableEditable<'a> {
    pub fn new(builder: &'a mut LayoutBuilder, var: Variable) -> Self {
        VariableEditable {
            builder: builder,
            var: var,
            val: 0.0,
            strength: STRONG,
        }
    }
    pub fn strength(mut self, strength: f64) -> Self {
        self.strength = strength;
        self
    }
    pub fn set(mut self, val: f64) -> Self {
        self.val = val;
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
    val: f64,
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
pub mod container;
pub mod constraint;
pub mod linear_layout;

pub use self::solver::LimnSolver;

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use layout::{LimnSolver, LayoutBuilder, LayoutVars, LayoutRef};
    use layout::constraint::*;
    use resources::{IdGen, WidgetId};
    use util::{Size, Point, Rect};

    #[test]
    fn one_widget() {
        let mut solver = LimnSolver::new();
        let mut id_gen = IdGen::new();
        let mut widget_map = HashMap::new();

        let mut widget = TestWidget::new(&mut id_gen, &mut widget_map);
        layout!(widget:
            top_left(Point::new(0.0, 0.0)),
            dimensions(Size::new(200.0, 200.0)),
        );
        solver.add_widget(widget.id, &None, widget.layout);

        assert!(get_layout(&mut solver, &widget_map) == hashmap!{
            widget.id => Rect::new(Point::new(0.0, 0.0), Size::new(200.0, 200.0)),
        });
    }
    #[test]
    fn grid() {
        let mut solver = LimnSolver::new();
        let mut id_gen = IdGen::new();
        let mut widget_map = HashMap::new();

        let mut widget_o = TestWidget::new(&mut id_gen, &mut widget_map);
        let mut widget_tl = TestWidget::new(&mut id_gen, &mut widget_map);
        let mut widget_bl = TestWidget::new(&mut id_gen, &mut widget_map);
        let mut widget_tr = TestWidget::new(&mut id_gen, &mut widget_map);
        let mut widget_br = TestWidget::new(&mut id_gen, &mut widget_map);
        layout!(widget_o:
            top_left(Point::new(0.0, 0.0)),
            dimensions(Size::new(300.0, 300.0)),
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
        solver.add_widget(widget_o.id, &Some("widget_o".to_owned()), widget_o.layout);
        solver.add_widget(widget_tl.id, &Some("widget_tl".to_owned()), widget_tl.layout);
        solver.add_widget(widget_tr.id, &Some("widget_tr".to_owned()), widget_tr.layout);
        solver.add_widget(widget_bl.id, &Some("widget_bl".to_owned()), widget_bl.layout);
        solver.add_widget(widget_br.id, &Some("widget_br".to_owned()), widget_br.layout);

        assert!(get_layout(&mut solver, &widget_map) == hashmap!{
            widget_o.id => Rect::new(Point::new(0.0, 0.0), Size::new(300.0, 300.0)),
            widget_tl.id => Rect::new(Point::new(0.0, 0.0), Size::new(150.0, 150.0)),
            widget_tr.id => Rect::new(Point::new(150.0, 0.0), Size::new(150.0, 150.0)),
            widget_bl.id => Rect::new(Point::new(0.0, 150.0), Size::new(150.0, 150.0)),
            widget_br.id => Rect::new(Point::new(150.0, 150.0), Size::new(150.0, 150.0)),
        });
    }

    // code below is used to create a test harness for creating layouts outside of the widget graph
    struct TestWidget {
        id: WidgetId,
        layout: LayoutBuilder,
    }
    impl TestWidget {
        fn new(id_gen: &mut IdGen<WidgetId>, widget_map: &mut HashMap<WidgetId, LayoutVars>) -> Self {
            let layout = LayoutBuilder::new();
            let id = id_gen.next();
            widget_map.insert(id, layout.vars.clone());
            TestWidget {
                id: id,
                layout: layout,
            }
        }
        fn layout(&mut self) -> &mut LayoutBuilder {
            &mut self.layout
        }
    }
    impl LayoutRef for TestWidget {
        fn layout_ref(&self) -> &LayoutVars {
            &self.layout.vars
        }
    }
    fn get_layout(solver: &mut LimnSolver, widget_map: &HashMap<WidgetId, LayoutVars>) -> HashMap<WidgetId, Rect> {
        let mut map = HashMap::new();
        for (widget_id, var, value) in solver.fetch_changes() {
            let rect = map.entry(widget_id).or_insert(Rect::zero());
            let vars = widget_map.get(&widget_id).unwrap();
            vars.update_bounds(var, value, rect);
        }
        map
    }
}
