#[macro_use]
extern crate limn;
#[macro_use]
extern crate limn_layout;
extern crate text_layout;
extern crate glutin;

mod util;

use std::collections::HashMap;

use text_layout::Align;

use limn::prelude::*;

use limn::widgets::button::PushButtonBuilder;
use limn::widgets::slider::{SliderBuilder, SetSliderValue};
use limn::draw::text::TextStyle;
use limn::draw::rect::{RectState, RectStyle};
use limn::draw::ellipse::{EllipseState, EllipseStyle};
use limn::widgets::edit_text::{self, TextUpdated};
use limn::widgets::text::TextBuilder;
use limn::input::keyboard::KeyboardInput;

struct SliderControl {
    widget: WidgetBuilder,
}

impl SliderControl {
    fn new() -> Self {
        let text_style = style!(TextStyle::TextColor: selector!(BLACK, INACTIVE: GRAY_50));
        let mut widget = WidgetBuilder::new("slider_container");
        widget.add_prop(Property::Inactive);
        let mut slider_title = TextBuilder::new_with_style(
            style!(parent: text_style, TextStyle::Text: "Circle Size".to_owned()));
        slider_title.set_name("slider_title");
        slider_title.layout().add(align_left(&widget));
        let mut slider_value = TextBuilder::new_with_style(
            style!(parent: text_style, TextStyle::Align: Align::End, TextStyle::Text: "30".to_owned()));
        slider_value
            .set_name("slider_value")
            .add_handler_fn(edit_text::text_change_handle);
        slider_value.layout().add(align_right(&widget));
        let mut slider_widget = SliderBuilder::new();
        slider_widget.set_name("slider_widget");
        slider_widget.layout().add(constraints![
            min_width(300.0),
            below(&slider_title).padding(10.0),
            below(&slider_value).padding(10.0),
            match_width(&widget),
        ]);

        let slider_value_ref = slider_value.widget_ref();
        slider_widget.on_value_changed(move |size, args| {
            let size = size * 100.0;
            slider_value_ref.event(TextUpdated((size as i32).to_string()));
            args.ui.event(CircleEvent::Resize(size));
        }).set_value(0.30);
        let slider_widget_ref = slider_widget.widget_ref();
        let slider_value_ref = slider_value.widget_ref();
        widget.add_handler_fn(move |event: &SetSliderValue, _| {
            let size = event.0;
            slider_widget_ref.event(SetSliderValue(size / 100.0));
            slider_value_ref.event(TextUpdated((size as i32).to_string()));
        });
        widget
            .add_child(slider_title)
            .add_child(slider_value)
            .add_child(slider_widget);
        SliderControl {
            widget: widget,
        }
    }
}
widget_wrapper!(SliderControl);

struct ControlBar {
    widget: WidgetBuilder,
    undo: WidgetRef,
    redo: WidgetRef,
    slider: WidgetRef,
}

impl ControlBar {
    fn new() -> Self {
        let control_color = GRAY_70;
        let mut widget = WidgetBuilder::new("control_bar");
        let style = style!(RectStyle::BackgroundColor: control_color);
        widget
            .set_draw_state_with_style(RectState::new(), style)
            .hbox(30.0);
        let mut undo_widget = PushButtonBuilder::new();
        undo_widget
            .set_text("Undo")
            .add_prop(Property::Inactive)
            .on_click(|_, args| { args.ui.event(CircleEvent::Undo); });
        undo_widget.layout().add(center_vertical(&widget));
        let mut redo_widget = PushButtonBuilder::new();
        redo_widget
            .set_text("Redo")
            .add_prop(Property::Inactive)
            .on_click(|_, args| { args.ui.event(CircleEvent::Redo); });
        redo_widget.layout().add(center_vertical(&widget));
        let slider_container = SliderControl::new();
        let slider_ref = slider_container.widget_ref();
        let (undo_ref, redo_ref) = (undo_widget.widget_ref(), redo_widget.widget_ref());
        widget
            .add_child(undo_widget)
            .add_child(redo_widget)
            .add_child(slider_container);
        ControlBar {
            widget: widget,
            undo: undo_ref,
            redo: redo_ref,
            slider: slider_ref,
        }
    }
}
widget_wrapper!(ControlBar);

fn create_circle(center: &Point, mut parent_ref: WidgetRef, size: f32) -> WidgetRef {
    let style = style!(EllipseStyle::BackgroundColor: selector!(WHITE, SELECTED: RED),
                       EllipseStyle::Border: Some((2.0, BLACK)));
    let mut widget = WidgetBuilder::new("circle");
    widget.set_draw_state_with_style(EllipseState::new(), style);
    widget.add_handler(CircleHandler { center: *center });
    let widget_ref = widget.widget_ref();
    let widget_ref_clone = widget.widget_ref();
    widget_ref.event(ResizeEvent(size));
    widget.on_click(move |_, args| {
        args.ui.event(CircleEvent::Select(Some(widget_ref.clone())));
    });
    parent_ref.add_child(widget);
    widget_ref_clone
}

struct ResizeEvent(f32);
struct CircleHandler {
    center: Point,
}
impl EventHandler<ResizeEvent> for CircleHandler {
    fn handle(&mut self, event: &ResizeEvent, args: EventArgs) {
        let radius = event.0 / 2.0;
        args.widget.update_layout(|layout| {
            layout.edit_top().set(self.center.y - radius);
            layout.edit_left().set(self.center.x - radius);
            layout.edit_right().set(self.center.x + radius);
            layout.edit_bottom().set(self.center.y + radius);
        });
    }
}

enum CircleEvent {
    Add(Point),
    Undo,
    Redo,
    Select(Option<WidgetRef>),
    Delete,
    Resize(f32),
}

struct CircleEventHandler {
    circle_canvas_ref: WidgetRef,
    undo_ref: WidgetRef,
    redo_ref: WidgetRef,
    slider_ref: WidgetRef,

    circles: HashMap<WidgetRef, (Point, f32)>,
    undo_queue: Vec<WidgetRef>,
    redo_queue: Vec<(Point, f32)>,
    selected: Option<WidgetRef>,
}
impl CircleEventHandler {
    fn new(circle_canvas_ref: WidgetRef, control_bar: &ControlBar) -> Self {
        CircleEventHandler {
            circle_canvas_ref: circle_canvas_ref,
            undo_ref: control_bar.undo.clone(),
            redo_ref: control_bar.redo.clone(),
            slider_ref: control_bar.slider.clone(),

            circles: HashMap::new(),
            undo_queue: Vec::new(),
            redo_queue: Vec::new(),
            selected: None,
        }
    }
}
impl EventHandler<CircleEvent> for CircleEventHandler {
    fn handle(&mut self, event: &CircleEvent, args: EventArgs) {
        match *event {
            CircleEvent::Add(point) => {
                let size = 30.0;
                let circle_ref = create_circle(&point, self.circle_canvas_ref.clone(), size);
                self.circles.insert(circle_ref.clone(), (point, size));
                self.undo_queue.push(circle_ref.clone());
                self.redo_queue.clear();

                self.undo_ref.event_subtree(PropChange::Remove(Property::Inactive));
                self.redo_ref.event_subtree(PropChange::Add(Property::Inactive));
                args.ui.event(CircleEvent::Select(Some(circle_ref)));
            }
            CircleEvent::Undo => {
                if !self.circles.is_empty() {
                    let mut widget_ref = self.undo_queue.pop().unwrap();
                    let (point, size) = self.circles.remove(&widget_ref).unwrap();
                    widget_ref.remove_widget();
                    self.redo_queue.push((point, size));

                    self.redo_ref.event_subtree(PropChange::Remove(Property::Inactive));
                    if self.circles.is_empty() {
                        self.undo_ref.event_subtree(PropChange::Add(Property::Inactive));
                    }
                }
            }
            CircleEvent::Redo => {
                if !self.redo_queue.is_empty() {
                    let (point, size) = self.redo_queue.pop().unwrap();
                    let circle_ref = create_circle(&point, self.circle_canvas_ref.clone(), size);
                    self.circles.insert(circle_ref.clone(), (point, size));
                    self.undo_queue.push(circle_ref);
                    if self.redo_queue.is_empty() {
                        self.redo_ref.event_subtree(PropChange::Add(Property::Inactive));
                    }
                }
            }
            CircleEvent::Select(ref new_selected) => {
                if let Some(ref selected) = self.selected {
                    selected.event_subtree(PropChange::Remove(Property::Selected));
                }
                self.selected = new_selected.clone();
                if let Some(ref selected) = self.selected {
                    let size = self.circles.get_mut(selected).unwrap().1;
                    self.slider_ref.event(SetSliderValue(size));
                    selected.event_subtree(PropChange::Add(Property::Selected));
                    self.slider_ref.event_subtree(PropChange::Remove(Property::Inactive));
                } else {
                    self.slider_ref.event_subtree(PropChange::Add(Property::Inactive));
                }
            }
            CircleEvent::Delete => {
                if let Some(mut selected) = self.selected.take() {
                    selected.remove_widget();
                    self.slider_ref.event_subtree(PropChange::Add(Property::Inactive));
                }
            }
            CircleEvent::Resize(size) => {
                if let Some(ref selected) = self.selected {
                    selected.event(ResizeEvent(size));
                    self.circles.get_mut(selected).unwrap().1 = size;
                }
            }
        }
    }
}

fn main() {
    let mut app = util::init_default("Limn circles demo");
    let mut root = WidgetBuilder::new("root");
    root.vbox();

    let mut circle_canvas = WidgetBuilder::new("circle_canvas");
    circle_canvas.no_container();
    circle_canvas.layout().add(constraints![
        match_width(&root),
        min_height(600.0)
    ]);
    circle_canvas
        .set_draw_state_with_style(RectState::new(), style!(RectStyle::BackgroundColor: WHITE))
        .on_click(|event, args| {
            let event = CircleEvent::Add(event.position);
            args.ui.event(event);
        });
    let mut control_bar = ControlBar::new();
    control_bar.layout().add(constraints![
        match_width(&root),
        align_bottom(&root),
        shrink_vertical(),
    ]);
    app.add_handler(CircleEventHandler::new(circle_canvas.widget_ref(), &control_bar));
    app.add_handler_fn(|event: &KeyboardInput, args| {
        if let KeyboardInput(glutin::ElementState::Released, _, Some(glutin::VirtualKeyCode::Delete)) = *event {
            args.ui.event(CircleEvent::Delete);
        }
    });
    root.add_child(circle_canvas);
    root.add_child(control_bar);
    app.main_loop(root);
}
