use std::collections::HashMap;

use cassowary::strength::*;
use cassowary::WeightedRelation::*;
use cassowary::{Variable, Constraint};

use super::{LayoutId, LayoutVars, Layout, LayoutContainer};
use super::constraint::*;

/// Specifies the extra space between elements along the primary axis
#[derive(Debug, Copy, Clone,PartialEq)]
pub enum Spacing {
    /// Equal spacing before, after and between all elements
    Around,
    /// Equal spacing between elements, with no extra space on the ends
    Between,
    /// Elements packed together at the start, leaving space at the end
    End,
    /// Elements packed together at the end, leaving space at the start
    Start,
}

/// Specifies alignment of items perpendicular to the primary axis
#[derive(Debug, Copy, Clone)]
pub enum ItemAlignment {
    /// No constraints are added, items can be aligned individually
    None,
    /// Item width/height matched layout parent
    Fill,
    /// Items centered within layout parent width/height
    Center,
    /// For a vertical layout, align items to the parent's left bound.
    /// Do not use in a horizontal layout
    Left,
    /// For a vertical layout, align items to the parent's right bound.
    /// Do not use in a horizontal layout
    Right,
    /// For a horizontal layout, align items to the parent's top bound.
    /// Do not use in a vertical layout
    Top,
    /// For a horizontal layout, align items to the parent's bottom bound.
    /// Do not use in a vertical layout
    Bottom,
}

#[derive(Debug, Copy, Clone)]
pub struct LinearLayoutSettings {
    /// Horizontal or vertical orientation of the current layout
    pub orientation: Orientation,
    /// Flex-like spacing property
    pub spacing: Spacing,
    /// Alignment of children
    pub item_align: ItemAlignment,
    /// Constrain items to have equal size along primary axis, and fill container
    pub fill_equal: bool,
    /// Padding around the current layout in relation to the parent item
    pub padding: f32,
}

impl LinearLayoutSettings {

    /// Creates a default `LinearLayoutSettings`, aligned to the
    /// end of the parent container
    pub fn new(orientation: Orientation) -> Self {
        LinearLayoutSettings {
            orientation: orientation,
            spacing: Spacing::End,
            item_align: ItemAlignment::None,
            fill_equal: false,
            padding: 0.0,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone)]
struct WidgetData {
    start: Variable,
    end: Variable,
    prev: Option<LayoutId>,
    next: Option<LayoutId>,
    end_constraint: Option<Constraint>,
}

pub struct LinearLayout {
    settings: LinearLayoutSettings,
    start: Variable,
    end: Variable,
    space: Variable,
    size: Option<Variable>,
    widgets: HashMap<LayoutId, WidgetData>,
    last_widget: Option<LayoutId>,
}

impl LinearLayout {
    pub fn new(parent: &mut Layout, settings: LinearLayoutSettings) -> Self {
        let start = Variable::new();
        let end = Variable::new();
        parent.add_associated_var(start, "linear_layout_start");
        parent.add_associated_var(end, "linear_layout_end");
        let space = Variable::new();
        parent.add_associated_var(space, "linear_layout_space");
        match settings.spacing {
            Spacing::Between | Spacing::Around => parent.add(space | GE(REQUIRED) | settings.padding),
            _ => parent.add(space | EQ(REQUIRED) | settings.padding)
        };
        let parent_start = beginning(settings.orientation, &parent.vars);
        let parent_end = ending(settings.orientation, &parent.vars);
        match settings.spacing {
            Spacing::Around => {
                parent.add(constraints![
                    start | EQ(REQUIRED) | parent_start + space,
                    end | EQ(REQUIRED) | parent_end - space,
                ]);
            },
            _ => {
                parent.add(constraints![
                    start | EQ(REQUIRED) | parent_start,
                    end | EQ(REQUIRED) | parent_end,
                ]);
            }
        }

        let size = if settings.fill_equal {
            let size = Variable::new();
            parent.add_associated_var(size, "linear_layout_size");
            let parent_size = axis_length(settings.orientation, &parent.vars);
            parent.add(parent_size | EQ(STRONG) | size);
            Some(size)
        } else {
            None
        };
        LinearLayout {
            settings: settings,
            start: start,
            end: end,
            space: space,
            size: size,
            widgets: HashMap::new(),
            last_widget: None,
        }
    }
}

impl LayoutContainer for LinearLayout {
    fn add_child(&mut self, parent: &mut Layout, child: &mut Layout) {

        let child_start = beginning(self.settings.orientation, &child.vars);
        let child_end = ending(self.settings.orientation, &child.vars);

        parent.add(child_start | GE(REQUIRED) | self.start);
        parent.add(child_end | LE(REQUIRED) | self.end);

        if let Some(last_id) = self.last_widget {
            let last_widget = self.widgets.get_mut(&last_id).unwrap();
            parent.add(child_start | EQ(REQUIRED) | last_widget.end + self.space);
            last_widget.next = Some(child.id);
        } else {
            if self.settings.spacing != Spacing::Start {
                parent.add(child_start | EQ(REQUIRED) | self.start);
            }
        }
        let end_constraint = {
            if self.settings.spacing != Spacing::End {
                if let Some(last_id) = self.last_widget {
                    let last_widget = self.widgets.get_mut(&last_id).unwrap();
                    parent.remove_constraint(last_widget.end_constraint.take().unwrap());
                }
                let end_constraint = child_end | EQ(REQUIRED) | self.end;
                parent.add(end_constraint.clone());
                Some(end_constraint)
            } else {
                None
            }
        };
        self.widgets.insert(child.id, WidgetData {
            start: child_start,
            end: child_end,
            prev: self.last_widget,
            next: None,
            end_constraint: end_constraint,
        });
        self.last_widget = Some(child.id);

        if self.settings.fill_equal {
            let child_size = axis_length(self.settings.orientation, &child.vars);
            parent.add(child_size | EQ(REQUIRED) | self.size.unwrap());
        }
        match self.settings.orientation {
            Orientation::Horizontal => {
                match self.settings.item_align {
                    ItemAlignment::Fill => {
                        child.add(constraints![
                            align_top(parent),
                            align_bottom(parent),
                        ]);
                    },
                    ItemAlignment::Center => {
                        child.add(constraints![
                            center_vertical(parent),
                            bound_top(parent),
                            bound_bottom(parent),
                        ]);
                    },
                    ItemAlignment::Top => {
                        child.add(constraints![
                            align_top(parent),
                            bound_bottom(parent),
                        ]);
                    },
                    ItemAlignment::Bottom => {
                        child.add(constraints![
                            bound_top(parent),
                            align_bottom(parent),
                        ]);
                    },
                    ItemAlignment::None => {
                        child.add(constraints![
                            bound_top(parent),
                            bound_bottom(parent),
                        ]);
                    },
                    _ => panic!("Invalid linear layout settings"),
                }
            },
            Orientation::Vertical => {
                match self.settings.item_align {
                    ItemAlignment::Fill => {
                        child.add(constraints![
                            align_left(parent),
                            align_right(parent),
                        ]);
                    },
                    ItemAlignment::Center => {
                        child.add(constraints![
                            center_horizontal(parent),
                            bound_left(parent),
                            bound_right(parent),
                        ]);
                    },
                    ItemAlignment::Left => {
                        child.add(constraints![
                            align_left(parent),
                            bound_right(parent),
                        ]);
                    },
                    ItemAlignment::Right => {
                        child.add(constraints![
                            bound_left(parent),
                            align_right(parent),
                        ]);
                    },
                    ItemAlignment::None => {
                        child.add(constraints![
                            bound_left(parent),
                            bound_right(parent),
                        ]);
                    },
                    _ => panic!("Invalid linear layout settings"),
                }
            }
        }
    }

    fn remove_child(&mut self, parent: &mut Layout, child: &mut Layout) {
        if let Some(widget_data) = self.widgets.remove(&child.id) {
            if let Some(prev) = widget_data.prev {
                let next_start = widget_data.next.map(|next_id| self.widgets[&next_id].start);
                let prev = self.widgets.get_mut(&prev).unwrap();
                if let Some(next_start) = next_start {
                    parent.add(next_start | EQ(REQUIRED) | prev.end + self.space);
                } else {
                    if self.settings.spacing != Spacing::End {
                        let end_constraint = prev.end | EQ(REQUIRED) | self.end;
                        parent.add(end_constraint.clone());
                        prev.end_constraint = Some(end_constraint);
                    }
                }
                prev.next = widget_data.next;
            } else if let Some(next) = widget_data.next {
                if self.settings.spacing != Spacing::Start {
                    let next_start = self.widgets[&next].start;
                    parent.add(next_start | EQ(REQUIRED) | self.start);
                }
            }
            if let Some(next) = widget_data.next {
                self.widgets.get_mut(&next).unwrap().prev = widget_data.prev;
            }

            if let Some(last_id) = self.last_widget {
                if last_id == child.id {
                    self.last_widget = widget_data.prev;
                }
            }
        }
    }
}

fn beginning(orientation: Orientation, layout: &LayoutVars) -> Variable {
    match orientation {
        Orientation::Horizontal => layout.left,
        Orientation::Vertical => layout.top,
    }
}
fn ending(orientation: Orientation, layout: &LayoutVars) -> Variable {
    match orientation {
        Orientation::Horizontal => layout.right,
        Orientation::Vertical => layout.bottom,
    }
}
fn axis_length(orientation: Orientation, layout: &LayoutVars) -> Variable {
    match orientation {
        Orientation::Horizontal => layout.width,
        Orientation::Vertical => layout.height,
    }
}
