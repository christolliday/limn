use cassowary::{Variable, Constraint};
use cassowary::WeightedRelation::*;
use cassowary::strength::*;

use widget::WidgetBuilder;
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
        let constraint = LinearLayout::beginning(self.orientation, &widget) | GE(REQUIRED) |
                         self.end;
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

pub struct LayoutVars {
    pub left: Variable,
    pub top: Variable,
    pub right: Variable,
    pub bottom: Variable,

    left_val: f64,
    top_val: f64,
    right_val: f64,
    bottom_val: f64,
}
impl LayoutVars {
    pub fn new() -> Self {
        LayoutVars {
            left: Variable::new(),
            top: Variable::new(),
            right: Variable::new(),
            bottom: Variable::new(),
            left_val: 0.0,
            top_val: 0.0,
            right_val: 0.0,
            bottom_val: 0.0,
        }
    }
    pub fn update_val(&mut self, var: Variable, value: f64) {
        if var == self.left {
            self.left_val = value;
        } else if var == self.top {
            self.top_val = value;
        } else if var == self.right {
            self.right_val = value;
        } else if var == self.bottom {
            self.bottom_val = value;
        } else {
            panic!();
        }
    }
    pub fn bounds(&self) -> Rectangle {
        Rectangle {
            left: self.left_val,
            top: self.top_val,
            width: self.right_val - self.left_val,
            height: self.bottom_val - self.top_val, 
        }
    }
    pub fn get_dims(&self) -> Dimensions {
        Dimensions {
            width: self.right_val - self.left_val,
            height: self.bottom_val - self.top_val,
        }
    }
}

pub struct LayoutBuilder {
    pub vars: LayoutVars,
    pub constraints: Vec<Constraint>,
}
impl LayoutBuilder {
    pub fn new() -> Self {
        LayoutBuilder {
            vars: LayoutVars::new(),
            constraints: Vec::new(),
        }
    }
    pub fn build(self) -> (LayoutVars, Vec<Constraint>) {
        (self.vars, self.constraints)
    }
    pub fn match_layout(&mut self, widget: &LayoutVars) {
        self.match_width(widget);
        self.match_height(widget);
    }
    pub fn match_width(&mut self, widget: &LayoutVars) {
        self.constraints.push(self.vars.left | EQ(REQUIRED) | widget.left);
        self.constraints.push(self.vars.right | EQ(REQUIRED) | widget.right);
    }
    pub fn match_height(&mut self, widget: &LayoutVars) {
        self.constraints.push(self.vars.top | EQ(REQUIRED) | widget.top);
        self.constraints.push(self.vars.bottom | EQ(REQUIRED) | widget.bottom);
    }
    pub fn width(&mut self, width: Scalar) {
        self.constraints.push(self.vars.right - self.vars.left | EQ(REQUIRED) | width);
    }
    pub fn height(&mut self, height: Scalar) {
        self.constraints.push(self.vars.bottom - self.vars.top | EQ(REQUIRED) | height);
    }
    pub fn min_width(&mut self, width: Scalar) {
        self.constraints.push(self.vars.right - self.vars.left | GE(REQUIRED) | width);
    }
    pub fn min_height(&mut self, height: Scalar) {
        self.constraints.push(self.vars.bottom - self.vars.top | GE(REQUIRED) | height);
    }
    pub fn dimensions(&mut self, dimensions: Dimensions) {
        self.width(dimensions.width);
        self.height(dimensions.height);
    }
    pub fn min_dimensions(&mut self, dimensions: Dimensions) {
        self.min_width(dimensions.width);
        self.min_height(dimensions.height);
    }
    pub fn shrink(&mut self) {
        self.width_strength(0.0, WEAK);
        self.height_strength(0.0, WEAK);
    }
    pub fn width_strength(&mut self, width: Scalar, strength: f64) {
        self.constraints
            .push(self.vars.right - self.vars.left | EQ(strength) | width);
    }
    pub fn height_strength(&mut self, height: Scalar, strength: f64) {
        self.constraints
            .push(self.vars.bottom - self.vars.top | EQ(strength) | height);
    }
    pub fn top_left(&mut self, point: Point, strength: Option<f64>) {
        self.constraints
            .push(self.vars.left | EQ(strength.unwrap_or(REQUIRED)) | point.x);
        self.constraints
            .push(self.vars.top | EQ(strength.unwrap_or(REQUIRED)) | point.y);
    }
    pub fn center(&mut self, widget: &LayoutVars) {
        self.center_horizontal(widget);
        self.center_vertical(widget);
    }
    pub fn center_horizontal(&mut self, widget: &LayoutVars) {
        self.constraints
            .push(self.vars.left - widget.left | EQ(REQUIRED) | widget.right - self.vars.right);
    }
    pub fn center_vertical(&mut self, widget: &LayoutVars) {
        self.constraints
            .push(self.vars.top - widget.top | EQ(REQUIRED) | widget.bottom - self.vars.bottom);
    }

    pub fn align_top(&mut self, widget: &LayoutVars, padding: Option<f64>) {
        self.constraints
            .push(self.vars.top - widget.top | EQ(REQUIRED) | padding.unwrap_or(0.0));
    }
    pub fn align_bottom(&mut self, widget: &LayoutVars, padding: Option<f64>) {
        self.constraints
            .push(widget.bottom - self.vars.bottom | EQ(REQUIRED) | padding.unwrap_or(0.0));
    }
    pub fn align_left(&mut self, widget: &LayoutVars, padding: Option<f64>) {
        self.constraints
            .push(self.vars.left - widget.left | EQ(REQUIRED) | padding.unwrap_or(0.0));
    }
    pub fn align_right(&mut self, widget: &LayoutVars, padding: Option<f64>) {
        self.constraints
            .push(widget.right - self.vars.right | EQ(REQUIRED) | padding.unwrap_or(0.0));
    }

    pub fn above(&mut self, widget: &LayoutVars, padding: Option<f64>) {
        self.constraints
            .push(self.vars.bottom - widget.top | GE(REQUIRED) | padding.unwrap_or(0.0));
    }
    pub fn below(&mut self, widget: &LayoutVars, padding: Option<f64>) {
        self.constraints
            .push(self.vars.top - widget.bottom | GE(REQUIRED) | padding.unwrap_or(0.0));
    }
    pub fn to_left_of(&mut self, widget: &LayoutVars, padding: Option<f64>) {
        self.constraints
            .push(widget.left - self.vars.right | GE(REQUIRED) | padding.unwrap_or(0.0));
    }
    pub fn to_right_of(&mut self, widget: &LayoutVars, padding: Option<f64>) {
        self.constraints
            .push(self.vars.left - widget.right | GE(REQUIRED) | padding.unwrap_or(0.0));
    }

    pub fn bound_left(&mut self, widget: &LayoutVars, padding: Option<f64>) {
        self.constraints
            .push(self.vars.left - widget.left | GE(REQUIRED) | padding.unwrap_or(0.0));
    }
    pub fn bound_top(&mut self, widget: &LayoutVars, padding: Option<f64>) {
        self.constraints
            .push(self.vars.top - widget.top | GE(REQUIRED) | padding.unwrap_or(0.0));
    }
    pub fn bound_right(&mut self, widget: &LayoutVars, padding: Option<f64>) {
        self.constraints
            .push(widget.right - self.vars.right | GE(REQUIRED) | padding.unwrap_or(0.0));
    }
    pub fn bound_bottom(&mut self, widget: &LayoutVars, padding: Option<f64>) {
        self.constraints.push(widget.bottom - self.vars.bottom | GE(REQUIRED) | padding.unwrap_or(0.0));
    }

    pub fn bound_by(&mut self, widget: &LayoutVars, padding: Option<f64>) {
        self.bound_left(widget, padding);
        self.bound_top(widget, padding);
        self.bound_right(widget, padding);
        self.bound_bottom(widget, padding);
    }

    pub fn scroll_inside(&mut self, widget: &LayoutVars) {
        self.constraints.push(self.vars.left | LE(REQUIRED) | widget.left);
        self.constraints.push(self.vars.top | LE(REQUIRED) | widget.top);
        // STRONG not REQUIRED because not satisfiable if layout is smaller than it's parent
        self.constraints.push(self.vars.right | GE(STRONG) | widget.right);
        self.constraints.push(self.vars.bottom | GE(STRONG) | widget.bottom);
    }
}
