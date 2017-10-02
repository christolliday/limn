use cassowary::{Variable, Constraint, Term, Expression};
use cassowary::WeightedRelation::*;
use cassowary::strength::*;

use super::{LAYOUT, LayoutRef, LayoutVars, Size, Point};

pub fn width(width: f32) -> WidgetConstraintBuilder {
    WidgetConstraint::Width(width).builder(REQUIRED)
}
pub fn height(height: f32) -> WidgetConstraintBuilder {
    WidgetConstraint::Height(height).builder(REQUIRED)
}
pub fn min_width(width: f32) -> WidgetConstraintBuilder {
    WidgetConstraint::MinWidth(width).builder(REQUIRED)
}
pub fn min_height(height: f32) -> WidgetConstraintBuilder {
    WidgetConstraint::MinHeight(height).builder(REQUIRED)
}
pub fn size(size: Size) -> WidgetConstraintBuilder {
    WidgetConstraint::Size(size).builder(REQUIRED)
}
pub fn min_size(size: Size) -> WidgetConstraintBuilder {
    WidgetConstraint::MinSize(size).builder(REQUIRED)
}
pub fn aspect_ratio(aspect_ratio: f32) -> WidgetConstraintBuilder {
    WidgetConstraint::AspectRatio(aspect_ratio).builder(REQUIRED)
}
pub fn shrink() -> WidgetConstraintBuilder {
    WidgetConstraint::Shrink.builder(WEAK)
}
pub fn shrink_horizontal() -> WidgetConstraintBuilder {
    WidgetConstraint::ShrinkHorizontal.builder(WEAK)
}
pub fn shrink_vertical() -> WidgetConstraintBuilder {
    WidgetConstraint::ShrinkVertical.builder(WEAK)
}
pub fn top_left(point: Point) -> WidgetConstraintBuilder {
    WidgetConstraint::TopLeft(point).builder(REQUIRED)
}
pub fn center<T: LayoutRef>(widget: &T) -> WidgetConstraintBuilder {
    WidgetConstraint::Center(widget.layout_ref().clone()).builder(REQUIRED)
}
pub fn center_horizontal<T: LayoutRef>(widget: &T) -> WidgetConstraintBuilder {
    let widget = widget.layout_ref();
    WidgetConstraint::CenterHorizontal(widget.left, widget.right).builder(REQUIRED)
}
pub fn center_vertical<T: LayoutRef>(widget: &T) -> WidgetConstraintBuilder {
    let widget = widget.layout_ref();
    WidgetConstraint::CenterVertical(widget.top, widget.bottom).builder(REQUIRED)
}

pub fn align_top<T: LayoutRef>(widget: &T) -> PaddableConstraintBuilder {
    let widget = widget.layout_ref();
    PaddableConstraint::AlignTop(widget.top).builder(REQUIRED)
}
pub fn align_bottom<T: LayoutRef>(widget: &T) -> PaddableConstraintBuilder {
    let widget = widget.layout_ref();
    PaddableConstraint::AlignBottom(widget.bottom).builder(REQUIRED)
}
pub fn align_left<T: LayoutRef>(widget: &T) -> PaddableConstraintBuilder {
    let widget = widget.layout_ref();
    PaddableConstraint::AlignLeft(widget.left).builder(REQUIRED)
}
pub fn align_right<T: LayoutRef>(widget: &T) -> PaddableConstraintBuilder {
    let widget = widget.layout_ref();
    PaddableConstraint::AlignRight(widget.right).builder(REQUIRED)
}

pub fn align_above<T: LayoutRef>(widget: &T) -> PaddableConstraintBuilder {
    let widget = widget.layout_ref();
    PaddableConstraint::AlignAbove(widget.top).builder(REQUIRED)
}
pub fn align_below<T: LayoutRef>(widget: &T) -> PaddableConstraintBuilder {
    let widget = widget.layout_ref();
    PaddableConstraint::AlignBelow(widget.bottom).builder(REQUIRED)
}
pub fn align_to_left_of<T: LayoutRef>(widget: &T) -> PaddableConstraintBuilder {
    let widget = widget.layout_ref();
    PaddableConstraint::AlignToLeftOf(widget.left).builder(REQUIRED)
}
pub fn align_to_right_of<T: LayoutRef>(widget: &T) -> PaddableConstraintBuilder {
    let widget = widget.layout_ref();
    PaddableConstraint::AlignToRightOf(widget.right).builder(REQUIRED)
}

pub fn above<T: LayoutRef>(widget: &T) -> PaddableConstraintBuilder {
    let widget = widget.layout_ref();
    PaddableConstraint::Above(widget.top).builder(REQUIRED)
}
pub fn below<T: LayoutRef>(widget: &T) -> PaddableConstraintBuilder {
    let widget = widget.layout_ref();
    PaddableConstraint::Below(widget.bottom).builder(REQUIRED)
}
pub fn to_left_of<T: LayoutRef>(widget: &T) -> PaddableConstraintBuilder {
    let widget = widget.layout_ref();
    PaddableConstraint::ToLeftOf(widget.left).builder(REQUIRED)
}
pub fn to_right_of<T: LayoutRef>(widget: &T) -> PaddableConstraintBuilder {
    let widget = widget.layout_ref();
    PaddableConstraint::ToRightOf(widget.right).builder(REQUIRED)
}

pub fn bound_left<T: LayoutRef>(outer: &T) -> PaddableConstraintBuilder {
    let outer = outer.layout_ref();
    PaddableConstraint::BoundLeft(outer.left).builder(REQUIRED)
}
pub fn bound_top<T: LayoutRef>(outer: &T) -> PaddableConstraintBuilder {
    let outer = outer.layout_ref();
    PaddableConstraint::BoundTop(outer.top).builder(REQUIRED)
}
pub fn bound_right<T: LayoutRef>(outer: &T) -> PaddableConstraintBuilder {
    let outer = outer.layout_ref();
    PaddableConstraint::BoundRight(outer.right).builder(REQUIRED)
}
pub fn bound_bottom<T: LayoutRef>(outer: &T) -> PaddableConstraintBuilder {
    let outer = outer.layout_ref();
    PaddableConstraint::BoundBottom(outer.bottom).builder(REQUIRED)
}

pub fn bound_by<T: LayoutRef>(outer: &T) -> PaddableConstraintBuilder {
    let outer = outer.layout_ref();
    PaddableConstraint::BoundBy(outer.clone()).builder(REQUIRED)
}

pub fn match_layout<T: LayoutRef>(widget: &T) -> PaddableConstraintBuilder {
    let widget = widget.layout_ref();
    PaddableConstraint::MatchLayout(widget.clone()).builder(REQUIRED)
}
pub fn match_width<T: LayoutRef>(widget: &T) -> PaddableConstraintBuilder {
    let widget = widget.layout_ref();
    PaddableConstraint::MatchWidth(widget.width).builder(REQUIRED)
}
pub fn match_height<T: LayoutRef>(widget: &T) -> PaddableConstraintBuilder {
    let widget = widget.layout_ref();
    PaddableConstraint::MatchHeight(widget.height).builder(REQUIRED)
}

#[derive(Clone)]
pub enum WidgetConstraint {
    Width(f32),
    Height(f32),
    MinWidth(f32),
    MinHeight(f32),
    Size(Size),
    MinSize(Size),
    AspectRatio(f32),
    Shrink,
    ShrinkHorizontal,
    ShrinkVertical,
    TopLeft(Point),
    Center(LayoutVars),
    CenterHorizontal(Variable, Variable),
    CenterVertical(Variable, Variable),
}

#[derive(Clone)]
pub enum PaddableConstraint {
    AlignTop(Variable),
    AlignBottom(Variable),
    AlignLeft(Variable),
    AlignRight(Variable),
    AlignAbove(Variable),
    AlignBelow(Variable),
    AlignToLeftOf(Variable),
    AlignToRightOf(Variable),
    Above(Variable),
    Below(Variable),
    ToLeftOf(Variable),
    ToRightOf(Variable),
    BoundLeft(Variable),
    BoundTop(Variable),
    BoundRight(Variable),
    BoundBottom(Variable),
    BoundBy(LayoutVars),
    MatchLayout(LayoutVars),
    MatchWidth(Variable),
    MatchHeight(Variable),
}
impl WidgetConstraint {
    pub fn builder(self, default_strength: f64) -> WidgetConstraintBuilder {
        WidgetConstraintBuilder {
            constraint: self,
            strength: default_strength,
        }
    }
}
impl PaddableConstraint {
    pub fn builder(self, default_strength: f64) -> PaddableConstraintBuilder {
        PaddableConstraintBuilder {
            constraint: self,
            strength: default_strength,
            padding: 0.0,
        }
    }
}

pub struct WidgetConstraintBuilder {
    constraint: WidgetConstraint,
    strength: f64,
}
impl WidgetConstraintBuilder {
    pub fn strength(mut self, strength: f64) -> Self {
        self.strength = strength;
        self
    }
}

pub struct PaddableConstraintBuilder {
    constraint: PaddableConstraint,
    strength: f64,
    padding: f32,
}
impl PaddableConstraintBuilder {
    pub fn strength(mut self, strength: f64) -> Self {
        self.strength = strength;
        self
    }
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }
}

pub trait ConstraintBuilder {
    fn build(&self, widget: &LayoutVars) -> Vec<Constraint>;
}

impl ConstraintBuilder for Constraint {
    fn build(&self, widget: &LayoutVars) -> Vec<Constraint> {
        let ref terms = self.expr().terms;
        let mut vars_replaced = false;
        let mut new_terms = Vec::new();
        for term in terms {
            let var = if term.variable == LAYOUT.left {
                widget.left
            } else if term.variable == LAYOUT.top {
                widget.top
            } else if term.variable == LAYOUT.right {
                widget.right
            } else if term.variable == LAYOUT.bottom {
                widget.bottom
            } else if term.variable == LAYOUT.width {
                widget.width
            } else if term.variable == LAYOUT.height {
                widget.height
            } else {
                term.variable
            };
            if var != term.variable {
                vars_replaced = true;
            }
            new_terms.push(Term {
                variable: var,
                coefficient: term.coefficient,
            });
        }
        if vars_replaced {
            let expr = Expression::new(new_terms, self.expr().constant);
            let cons = Constraint::new(expr, self.op(), self.strength());
            vec![ cons ]
        } else {
            // ensure hash value (from pointer) is unchanged if terms unchanged
            vec![ self.clone() ]
        }
    }
}

impl ConstraintBuilder for WidgetConstraintBuilder {
    fn build(&self, widget: &LayoutVars) -> Vec<Constraint> {
        let strength = self.strength;
        match self.constraint.clone() {
            WidgetConstraint::Width(width) => {
                vec![ widget.width | EQ(strength) | width ]
            }
            WidgetConstraint::Height(height) => {
                vec![ widget.height | EQ(strength) | height ]
            }
            WidgetConstraint::MinWidth(width) => {
                vec![ widget.width | GE(strength) | width ]
            }
            WidgetConstraint::MinHeight(height) => {
                vec![ widget.height | GE(strength) | height ]
            }
            WidgetConstraint::Size(size) => {
                vec![
                    widget.width | EQ(strength) | size.width,
                    widget.height | EQ(strength) | size.height,
                ]
            }
            WidgetConstraint::MinSize(size) => {
                vec![
                    widget.width | GE(strength) | size.width,
                    widget.height | GE(strength) | size.height,
                ]
            }
            WidgetConstraint::AspectRatio(aspect_ratio) => {
                vec![ aspect_ratio * widget.width | EQ(strength) | widget.height ]
            }
            WidgetConstraint::Shrink => {
                vec![
                    widget.width | EQ(strength) | 0.0,
                    widget.height | EQ(strength) | 0.0,
                ]
            }
            WidgetConstraint::ShrinkHorizontal => {
                vec![ widget.width | EQ(strength) | 0.0 ]
            }
            WidgetConstraint::ShrinkVertical => {
                vec![ widget.height | EQ(strength) | 0.0 ]
            }
            WidgetConstraint::TopLeft(point) => {
                vec![
                    widget.left | EQ(strength) | point.x,
                    widget.top | EQ(strength) | point.y,
                ]
            }
            WidgetConstraint::Center(other) => {
                vec![
                    widget.left - other.left | EQ(REQUIRED) | other.right - widget.right,
                    widget.top - other.top | EQ(REQUIRED) | other.bottom - widget.bottom,
                ]
            }
            WidgetConstraint::CenterHorizontal(left, right) => {
                vec![ widget.left - left | EQ(REQUIRED) | right - widget.right ]
            }
            WidgetConstraint::CenterVertical(top, bottom) => {
                vec![ widget.top - top | EQ(REQUIRED) | bottom - widget.bottom ]
            }
        }
    }
}

impl ConstraintBuilder for PaddableConstraintBuilder {
    fn build(&self, widget: &LayoutVars) -> Vec<Constraint> {
        let strength = self.strength;
        let padding = self.padding;
        match self.constraint.clone() {
            PaddableConstraint::AlignTop(top) => {
                vec![ widget.top - top | EQ(strength) | padding ]
            }
            PaddableConstraint::AlignBottom(bottom) => {
                vec![ bottom - widget.bottom | EQ(strength) | padding ]
            }
            PaddableConstraint::AlignLeft(left) => {
                vec![ widget.left - left | EQ(strength) | padding ]
            }
            PaddableConstraint::AlignRight(right) => {
                vec![ right - widget.right | EQ(strength) | padding ]
            }
            PaddableConstraint::AlignAbove(top) => {
                vec![ top - widget.bottom | EQ(strength) | padding ]
            }
            PaddableConstraint::AlignBelow(bottom) => {
                vec![ widget.top - bottom | EQ(strength) | padding ]
            }
            PaddableConstraint::AlignToLeftOf(left) => {
                vec![ left - widget.right | EQ(strength) | padding ]
            }
            PaddableConstraint::AlignToRightOf(right) => {
                vec![ widget.left - right | EQ(strength) | padding ]
            }
            PaddableConstraint::Above(top) => {
                vec![ top - widget.bottom | GE(strength) | padding ]
            }
            PaddableConstraint::Below(bottom) => {
                vec![ widget.top - bottom | GE(strength) | padding ]
            }
            PaddableConstraint::ToLeftOf(left) => {
                vec![ left - widget.right | GE(strength) | padding ]
            }
            PaddableConstraint::ToRightOf(right) => {
                vec![ widget.left - right | GE(strength) | padding ]
            }
            PaddableConstraint::BoundLeft(left) => {
                vec![ widget.left - left | GE(strength) | padding ]
            }
            PaddableConstraint::BoundTop(top) => {
                vec![ widget.top - top | GE(strength) | padding ]
            }
            PaddableConstraint::BoundRight(right) => {
                vec![ right - widget.right | GE(strength) | padding ]
            }
            PaddableConstraint::BoundBottom(bottom) => {
                vec![ bottom - widget.bottom | GE(strength) | padding ]
            }
            PaddableConstraint::BoundBy(other) => {
                vec![
                    widget.left - other.left | GE(strength) | padding,
                    widget.top - other.top | GE(strength) | padding,
                    other.right - widget.right | GE(strength) | padding,
                    other.bottom - widget.bottom | GE(strength) | padding,
                ]
            }
            PaddableConstraint::MatchLayout(other) => {
                vec![
                    widget.left - other.left | EQ(strength) | padding,
                    widget.top - other.top | EQ(strength) | padding,
                    other.right - widget.right | EQ(strength) | padding,
                    other.bottom - widget.bottom | EQ(strength) | padding,
                ]
            }
            PaddableConstraint::MatchWidth(width) => {
                vec![ width - widget.width | EQ(strength) | padding ]
            }
            PaddableConstraint::MatchHeight(height) => {
                vec![ height - widget.height | EQ(strength) | padding ]
            }
        }
    }
}

impl <C: ConstraintBuilder> ConstraintBuilder for Vec<C> {
    fn build(&self, widget: &LayoutVars) -> Vec<Constraint> {
        let mut constraints = Vec::new();
        for builder in self {
            constraints.extend(builder.build(widget));
        }
        constraints
    }
}

impl ConstraintBuilder for Box<ConstraintBuilder> {
    fn build(&self, widget: &LayoutVars) -> Vec<Constraint> {
        self.as_ref().build(widget)
    }
}
