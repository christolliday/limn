// Enable clippy if our Cargo.toml file asked us to do so.
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

#![warn(missing_copy_implementations,
        trivial_numeric_casts,
        trivial_casts,
        unused_extern_crates,
        unused_import_braces,
        unused_qualifications)]
#![cfg_attr(feature="clippy", warn(cast_possible_truncation))]
#![cfg_attr(feature="clippy", warn(cast_possible_wrap))]
#![cfg_attr(feature="clippy", warn(cast_precision_loss))]
#![cfg_attr(feature="clippy", warn(cast_sign_loss))]
#![cfg_attr(feature="clippy", warn(missing_docs_in_private_items))]
#![cfg_attr(feature="clippy", warn(mut_mut))]

// Disallow `println!`. Use `debug!` for debug output
// (which is provided by the `log` crate).
#![cfg_attr(feature="clippy", warn(print_stdout))]

// This allows us to use `unwrap` on `Option` values (because doing makes
// working with Regex matches much nicer) and when compiling in test mode
// (because using it in tests is idiomatic).
#![cfg_attr(all(not(test), feature="clippy"), warn(result_unwrap_used))]
#![cfg_attr(feature="clippy", warn(unseparated_literal_suffix))]
#![cfg_attr(feature="clippy", warn(wrong_pub_self_convention))]

#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
extern crate cassowary;
extern crate euclid;

use std::collections::HashSet;
use std::ops::Drop;
use std::mem;
use std::rc::Rc;
use std::cell::RefCell;

use cassowary::{Variable, Constraint};
use cassowary::WeightedRelation::*;
use cassowary::strength::*;

use euclid::{Point2D, Size2D, UnknownUnit};

use self::constraint::ConstraintBuilder;
use self::constraint::*;

pub type Length = euclid::Length<f32, UnknownUnit>;
pub type Size = Size2D<f32>;
pub type Point = Point2D<f32>;
pub type Rect = euclid::Rect<f32>;

pub type LayoutId = usize;

#[derive(Debug, Copy, Clone)]
pub struct LayoutVars {
    pub left: Variable,
    pub top: Variable,
    pub right: Variable,
    pub bottom: Variable,
    pub width: Variable,
    pub height: Variable,
}

impl LayoutVars {

    /// Creates a new set of empty `LayoutVars`
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

    /// Returns the current inner state of this struct as an array
    pub fn array(&self) -> [Variable; 6] {
        [self.left,
         self.top,
         self.right,
         self.bottom,
         self.width,
         self.height]
    }

    /// Checks if another `Variable` is the same as the
    pub fn var_type(&self, var: Variable) -> VarType {
        if var == self.left { VarType::Left }
        else if var == self.top { VarType::Top }
        else if var == self.right { VarType::Right }
        else if var == self.bottom { VarType::Bottom }
        else if var == self.width { VarType::Width }
        else if var == self.height { VarType::Height }
        else { VarType::Other }
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
    Other,
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
    container: Option<Rc<RefCell<LayoutContainer>>>,
    parent: Option<LayoutId>,
    children: Vec<LayoutId>,
    edit_vars: Vec<EditVariable>,
    constraints: HashSet<Constraint>,
    new_constraints: HashSet<Constraint>,
    removed_constraints: Vec<Constraint>,
    removed_children: Vec<LayoutId>,
    associated_vars: Vec<(Variable, String)>,
    pub hidden: bool,
}

impl Layout {

    /// Creates a new `Layout`.
    pub fn new(id: LayoutId, name: Option<String>) -> Self {
        let vars = LayoutVars::new();
        let mut new_constraints = HashSet::new();
        new_constraints.insert(vars.right - vars.left| EQ(REQUIRED) | vars.width);
        new_constraints.insert(vars.bottom - vars.top | EQ(REQUIRED) | vars.height);
        new_constraints.insert(vars.width | GE(REQUIRED) | 0.0);
        new_constraints.insert(vars.height | GE(REQUIRED) | 0.0);
        Layout {
            vars: vars,
            name: name,
            id: id,
            container: Some(Rc::new(RefCell::new(Frame::default()))),
            parent: None,
            children: Vec::new(),
            edit_vars: Vec::new(),
            constraints: HashSet::new(),
            new_constraints: new_constraints,
            removed_constraints: Vec::new(),
            removed_children: Vec::new(),
            associated_vars: Vec::new(),
            hidden: false,
        }
    }

    pub fn layout(&mut self) -> &mut Self {
        self
    }

    /// Clears the container of the current `Layout`.
    pub fn no_container(&mut self) {
        self.container = None;
    }

    /// Replaces the container of the current layout
    pub fn set_container<T>(&mut self, container: T) where T: LayoutContainer + 'static {
        self.container = Some(Rc::new(RefCell::new(container)));
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
    pub fn create_constraint<B: ConstraintBuilder>(&self, builder: B) -> Vec<Constraint> {
        builder.build(&self.vars)
    }
    pub fn add<B: ConstraintBuilder>(&mut self, builder: B) {
        let new_constraints = builder.build(&self.vars);
        self.new_constraints.extend(new_constraints);
    }
    pub fn remove_constraint(&mut self, constraint: Constraint) {
        if !self.new_constraints.remove(&constraint) {
            self.removed_constraints.push(constraint);
        }
    }
    pub fn remove_constraints(&mut self, constraints: Vec<Constraint>) {
        for constraint in constraints {
            if !self.new_constraints.remove(&constraint) {
                self.removed_constraints.push(constraint);
            }
        }
    }
    pub fn has_constraint(&mut self, constraints: &Vec<Constraint>) -> bool {
        for constraint in constraints {
            if self.new_constraints.contains(constraint) || self.constraints.contains(constraint) {
                return true
            }
        }
        false
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
    pub fn add_child(&mut self, child: &mut Layout) {
        child.parent = Some(self.id);
        self.children.push(child.id);
        if let Some(container) = self.container.clone() {
            container.borrow_mut().add_child(self, child);
        }
    }
    pub fn remove_child(&mut self, child: &mut Layout) {
        if let Some(container) = self.container.clone() {
            container.borrow_mut().remove_child(self, child);
        }
        if let Some(pos) = self.children.iter().position(|id| child.id == *id) {
            self.children.remove(pos);
        }
        self.removed_children.push(child.id);
    }
    pub fn get_removed_children(&mut self) -> Vec<LayoutId> {
        mem::replace(&mut self.removed_children, Vec::new())
    }
    pub fn get_children(&self) -> &Vec<LayoutId> {
        &self.children
    }
    pub fn add_associated_vars(&mut self, vars: &LayoutVars, name: &str) {
        for var in vars.array().iter() {
            let var_type = format!("{:?}", vars.var_type(*var)).to_lowercase();
            self.associated_vars.push((*var, format!("{}.{}", name, var_type)));
        }
    }
    pub fn add_associated_var(&mut self, var: Variable, name: &str) {
        self.associated_vars.push((var, name.to_owned()));
    }
    pub fn get_associated_vars(&mut self) -> Vec<(Variable, String)> {
        mem::replace(&mut self.associated_vars, Vec::new())
    }
    pub fn hide(&mut self) {
        self.hidden = true;
    }
    pub fn show(&mut self) {
        self.hidden = false;
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

#[derive(Debug, Copy, Clone)]
pub struct EditVariable {
    var: Variable,
    val: f64,
    strength: f64,
}

impl EditVariable {
    fn new(editable: &VariableEditable) -> Self {
        EditVariable {
            var: editable.var,
            val: editable.val.unwrap_or(0.0),
            strength: editable.strength,
        }
    }
}

/// Used to specify a list of constraints.
// Needed to box different ConstraintBuilder impls,
// can't be done without specifying Vec<Box<ConstraintBuilder>>.
// Can be removed if/when variadic generics are added to rust.
#[macro_export]
macro_rules! constraints {
    ( $ ( $ x : expr ) , * ) => {
        constraints!( $ ( $ x , ) * )
    };
    ( $ ( $ x : expr , ) * ) => {
        {
            let mut vec: Vec<Box<ConstraintBuilder>> = Vec::new();
            $(
                vec.push(Box::new($x));
            )*
            vec
        }
    };
}

pub trait LayoutContainer {
    fn add_child(&mut self, parent: &mut Layout, child: &mut Layout);
    fn remove_child(&mut self, _: &mut Layout, _: &mut Layout) {}
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Frame {
    padding: f32,
}

impl LayoutContainer for Frame {

    /// Adds a weak constraint to a Frame
    fn add_child(&mut self, parent: &mut Layout, child: &mut Layout) {
        child.add(constraints![
            bound_by(&parent).padding(self.padding),
            match_layout(&parent).strength(WEAK),
        ]);
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ExactFrame;

impl LayoutContainer for ExactFrame {
    fn add_child(&mut self, parent: &mut Layout, child: &mut Layout) {
        child.add(match_layout(&parent));
    }
}

pub mod solver;
pub mod constraint;
pub mod linear_layout;
pub mod grid_layout;

pub use self::solver::LimnSolver;

lazy_static! {
    pub static ref LAYOUT: LayoutVars = LayoutVars::new();
}
