use cassowary::{Variable, Constraint};
use cassowary::WeightedRelation::*;
use cassowary::strength::*;

use layout::LimnSolver;
use widget::builder::WidgetBuilder;
use resources::WidgetId;
use util::{Point, Rectangle, Dimensions, Scalar};

#[derive(Copy, Clone)]
pub enum Orientation {
    Horizontal,
    Vertical,
}
pub struct LinearLayout {
    pub orientation: Orientation,
    pub end: Variable,
    pub prev_id: WidgetId,
}
impl LinearLayout {
    pub fn new(orientation: Orientation, parent: &WidgetBuilder) -> Self {
        LinearLayout {
            orientation: orientation,
            end: LinearLayout::beginning(orientation, &parent),
            prev_id: parent.id,
        }
    }
    pub fn beginning(orientation: Orientation, widget: &WidgetBuilder) -> Variable {
        match orientation {
            Orientation::Horizontal => widget.layout.vars.left,
            Orientation::Vertical => widget.layout.vars.top,
        }
    }
    pub fn ending(orientation: Orientation, widget: &WidgetBuilder) -> Variable {
        match orientation {
            Orientation::Horizontal => widget.layout.vars.right,
            Orientation::Vertical => widget.layout.vars.bottom,
        }
    }
    pub fn add_widget(&mut self, widget: &mut WidgetBuilder) {
        let constraint = LinearLayout::beginning(self.orientation, &widget) | GE(REQUIRED) | self.end;
        let constraint = WidgetConstraint::Relative(constraint, self.prev_id);
        self.end = LinearLayout::ending(self.orientation, &widget);
        widget.layout.constraints.push(constraint);
        self.prev_id = widget.id;
    }
}

/*
constraint types, for declarative layout
pub enum ConstraintType {
    // EQ, StrDefault: WEAK
    ShrinkWidth,
    ShrinkHeight,
    // EQ, StrDefault: WEAK,
    GrowWidth,
    GrowHeight,
    // EQ, Args(Widget), StrDefault: REQ
    MatchWidth,
    MatchHeight,
    // GE, StrDefault: REQ
    MinWidth,
    MinHeight,
    // GE, StrDefault: REQ,
    MaxWidth,
    MaxHeight,
    // EQ, Args(Widget), StrDefault: REQ
    CenterHorizontal,
    CenterVertical,
    // EQ, Args(Widget, Option<padding>), StrDefault: REQ
    AlignTop,
    AlignBottom,
    AlignLeft,
    AlignRight,
    // GE, Args(Widget, Option<padding>), StrDefault: REQ
    BoundTop,
    BoundBottom,
    BoundLeft,
    BoundRight,
    // GE, Args(Widget, Option<padding>), StrDefault: REQ
    Above,
    Below,
    ToRightOf,
    ToLeftOf,
}
*/

pub enum WidgetConstraint {
    Local(Constraint),
    Relative(Constraint, WidgetId),
}

pub struct LayoutVars {
    pub left: Variable,
    pub top: Variable,
    pub right: Variable,
    pub bottom: Variable,
    pub bounds: Rectangle,
}
impl LayoutVars {
    pub fn new() -> Self {
        LayoutVars {
            left: Variable::new(),
            top: Variable::new(),
            right: Variable::new(),
            bottom: Variable::new(),
            bounds: Rectangle::new_empty(),
        }
    }
    pub fn update(&mut self, solver: &mut LimnSolver) {
        self.bounds = Rectangle {
            left: solver.get_value(self.left),
            top: solver.get_value(self.top),
            width: solver.get_value(self.right) - solver.get_value(self.left),
            height: solver.get_value(self.bottom) - solver.get_value(self.top),
        }
    }
    pub fn bounds(&self) -> Rectangle {
        self.bounds
    }
    pub fn get_dims(&self) -> Dimensions {
        Dimensions {
            width: self.bounds.width,
            height: self.bounds.height,
        }
    }
}

pub struct LayoutBuilder {
    pub vars: LayoutVars,
    pub constraints: Vec<WidgetConstraint>,
}
impl LayoutBuilder {
    pub fn new() -> Self {
        LayoutBuilder {
            vars: LayoutVars::new(),
            constraints: Vec::new(),
        }
    }
    pub fn build(self) -> (LayoutVars, Vec<WidgetConstraint>) {
        (self.vars, self.constraints)
    }
    pub fn match_layout(&mut self, widget: &WidgetBuilder) {
        self.match_width(widget);
        self.match_height(widget);
    }
    pub fn match_width(&mut self, widget: &WidgetBuilder) {
        self.constraints.push(WidgetConstraint::Relative(self.vars.left | EQ(REQUIRED) | widget.layout.vars.left, widget.id));
        self.constraints.push(WidgetConstraint::Relative(self.vars.right | EQ(REQUIRED) | widget.layout.vars.right, widget.id));
    }
    pub fn match_height(&mut self, widget: &WidgetBuilder) {
        self.constraints.push(WidgetConstraint::Relative(self.vars.top | EQ(REQUIRED) | widget.layout.vars.top, widget.id));
        self.constraints.push(WidgetConstraint::Relative(self.vars.bottom | EQ(REQUIRED) | widget.layout.vars.bottom, widget.id));
    }
    pub fn width(&mut self, width: Scalar) {
        self.constraints.push(WidgetConstraint::Local(self.vars.right - self.vars.left | EQ(REQUIRED) | width));
    }
    pub fn height(&mut self, height: Scalar) {
        self.constraints.push(WidgetConstraint::Local(self.vars.bottom - self.vars.top | EQ(REQUIRED) | height));
    }
    pub fn dimensions(&mut self, dimensions: Dimensions) {
        self.width(dimensions.width);
        self.height(dimensions.height);
    }
    pub fn minimize(&mut self) {
        self.width_strength(0.0, WEAK);
        self.height_strength(0.0, WEAK);
    }
    pub fn maximize(&mut self) {
        self.width_strength(10000.0, WEAK);
        self.height_strength(10000.0, WEAK);
    }
    pub fn width_strength(&mut self, width: Scalar, strength: f64) {
        self.constraints.push(WidgetConstraint::Local(self.vars.right - self.vars.left | EQ(strength) | width));
    }
    pub fn height_strength(&mut self, height: Scalar, strength: f64) {
        self.constraints.push(WidgetConstraint::Local(self.vars.bottom - self.vars.top | EQ(strength) | height));
    }
    pub fn top_left(&mut self, point: Point, strength: Option<f64>) {
        self.constraints.push(WidgetConstraint::Local(self.vars.left | EQ(strength.unwrap_or(REQUIRED)) | point.x));
        self.constraints.push(WidgetConstraint::Local(self.vars.top | EQ(strength.unwrap_or(REQUIRED)) | point.y));
    }
    pub fn center(&mut self, widget: &WidgetBuilder) {
        self.center_horizontal(widget);
        self.center_vertical(widget);
    }
    pub fn center_horizontal(&mut self, widget: &WidgetBuilder) {
        self.constraints.push(WidgetConstraint::Relative(self.vars.left - widget.layout.vars.left | EQ(REQUIRED) | widget.layout.vars.right - self.vars.right, widget.id));
    }
    pub fn center_vertical(&mut self, widget: &WidgetBuilder) {
        self.constraints.push(WidgetConstraint::Relative(self.vars.top - widget.layout.vars.top | EQ(REQUIRED) | widget.layout.vars.bottom - self.vars.bottom, widget.id));
    }

    pub fn align_top(&mut self, widget: &WidgetBuilder, padding: Option<f64>) {
        self.constraints.push(WidgetConstraint::Relative(self.vars.top - widget.layout.vars.top | EQ(REQUIRED) | padding.unwrap_or(0.0), widget.id));
    }
    pub fn align_bottom(&mut self, widget: &WidgetBuilder, padding: Option<f64>) {
        self.constraints.push(WidgetConstraint::Relative(widget.layout.vars.bottom - self.vars.bottom | EQ(REQUIRED) | padding.unwrap_or(0.0), widget.id));
    }
    pub fn align_left(&mut self, widget: &WidgetBuilder, padding: Option<f64>) {
        self.constraints.push(WidgetConstraint::Relative(self.vars.left - widget.layout.vars.left | EQ(REQUIRED) | padding.unwrap_or(0.0), widget.id));
    }
    pub fn align_right(&mut self, widget: &WidgetBuilder, padding: Option<f64>) {
        self.constraints.push(WidgetConstraint::Relative(widget.layout.vars.right - self.vars.right | EQ(REQUIRED) | padding.unwrap_or(0.0), widget.id));
    }
    
    pub fn above(&mut self, widget: &WidgetBuilder, padding: Option<f64>) {
        self.constraints.push(WidgetConstraint::Relative(self.vars.bottom - widget.layout.vars.top | GE(REQUIRED) | padding.unwrap_or(0.0), widget.id));
    }
    pub fn below(&mut self, widget: &WidgetBuilder, padding: Option<f64>) {
        self.constraints.push(WidgetConstraint::Relative(self.vars.top - widget.layout.vars.bottom | GE(REQUIRED) | padding.unwrap_or(0.0), widget.id));
    }
    pub fn to_left_of(&mut self, widget: &WidgetBuilder, padding: Option<f64>) {
        self.constraints.push(WidgetConstraint::Relative(widget.layout.vars.left - self.vars.right | GE(REQUIRED) | padding.unwrap_or(0.0), widget.id));
    }
    pub fn to_right_of(&mut self, widget: &WidgetBuilder, padding: Option<f64>) {
        self.constraints.push(WidgetConstraint::Relative(self.vars.left - widget.layout.vars.right | GE(REQUIRED) | padding.unwrap_or(0.0), widget.id));
    }

    pub fn bound_left(&mut self, widget: &WidgetBuilder, padding: Option<f64>) {
        self.constraints.push(WidgetConstraint::Relative(self.vars.left - widget.layout.vars.left | GE(REQUIRED) | padding.unwrap_or(0.0), widget.id));
    }
    pub fn bound_top(&mut self, widget: &WidgetBuilder, padding: Option<f64>) {
        self.constraints.push(WidgetConstraint::Relative(self.vars.top - widget.layout.vars.top | GE(REQUIRED) | padding.unwrap_or(0.0), widget.id));
    }
    pub fn bound_right(&mut self, widget: &WidgetBuilder, padding: Option<f64>) {
        self.constraints.push(WidgetConstraint::Relative(widget.layout.vars.right - self.vars.right| GE(REQUIRED) | padding.unwrap_or(0.0), widget.id));
    }
    pub fn bound_bottom(&mut self, widget: &WidgetBuilder, padding: Option<f64>) {
        self.constraints.push(WidgetConstraint::Relative(widget.layout.vars.bottom - self.vars.bottom | GE(REQUIRED) | padding.unwrap_or(0.0), widget.id));
    }

    pub fn bound_by(&mut self, widget: &WidgetBuilder, padding: Option<f64>) {
        self.bound_left(widget, padding);
        self.bound_top(widget, padding);
        self.bound_right(widget, padding);
        self.bound_bottom(widget, padding);
    }

    pub fn scroll_inside(&mut self, widget: &WidgetBuilder) {
        self.constraints.push(WidgetConstraint::Relative(self.vars.left | LE(REQUIRED) | widget.layout.vars.left, widget.id));
        self.constraints.push(WidgetConstraint::Relative(self.vars.top | LE(REQUIRED) | widget.layout.vars.top, widget.id));
        // STRONG not REQUIRED because not satisfiable if layout is smaller than it's parent
        self.constraints.push(WidgetConstraint::Relative(self.vars.right | GE(STRONG) | widget.layout.vars.right, widget.id));
        self.constraints.push(WidgetConstraint::Relative(self.vars.bottom | GE(STRONG) | widget.layout.vars.bottom, widget.id));
    }
}