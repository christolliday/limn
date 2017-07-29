#[macro_use]
extern crate limn;
#[macro_use]
extern crate limn_layout;
extern crate text_layout;
extern crate glutin;
extern crate graphics;
extern crate cassowary;

mod util;

use std::collections::HashMap;

use text_layout::Align;

use limn::prelude::*;

use limn::widgets::button::{PushButtonBuilder, WidgetClickable};
use limn::widgets::slider::{SliderBuilder, SetSliderValue};
use limn::drawable::text::{TextDrawable, TextStyleable};
use limn::drawable::rect::{RectDrawable, RectStyleable};
use limn::drawable::ellipse::{EllipseDrawable, EllipseStyleable};
use limn::widgets::edit_text::{self, TextUpdated};
use limn::input::keyboard::KeyboardInput;

fn create_slider_control() -> WidgetBuilder {

    let text_style = style!(TextStyleable::TextColor: selector!(BLACK, INACTIVE: [0.5, 0.5, 0.5, 1.0]));
    let mut slider_container = WidgetBuilder::new();
    slider_container
        .set_debug_name("slider_container")
        .set_inactive();
    let mut slider_title = WidgetBuilder::new();
    slider_title.set_debug_name("slider_title");
    slider_title.set_drawable_with_style(TextDrawable::new("Circle Size"), text_style.clone());
    layout!(slider_title:
        align_left(&slider_container),
        width(150.0));
    let style = style!(parent: text_style, TextStyleable::Align: Align::End);
    let mut slider_value = WidgetBuilder::new();
    slider_value
        .set_debug_name("slider_value")
        .set_drawable_with_style(TextDrawable::new("30"), style)
        .add_handler_fn(edit_text::text_change_handle);
    layout!(slider_value:
        width(50.0),
        align_right(&slider_container));
    let mut slider_widget = SliderBuilder::new();
    slider_widget.set_debug_name("slider_widget");
    layout!(slider_widget:
        width(300.0),
        below(&slider_title).padding(10.0),
        below(&slider_value).padding(10.0),
        match_width(&slider_container));

    let (slider_ref, slider_value_ref) = (slider_widget.widget.widget.clone(), slider_value.widget.clone());
    slider_widget.on_value_changed(move |size, _| {
        let size = size * 100.0;
        slider_value_ref.event(TextUpdated((size as i32).to_string()));
        event!(Target::Ui, CircleEvent::Resize(size));
    }).set_value(0.30);
    let slider_value_ref = slider_value.widget.clone();
    slider_container.add_handler_fn(move |event: &SetSliderValue, _| {
        let size = event.0;
        slider_ref.event(SetSliderValue(size / 100.0));
        slider_value_ref.event(TextUpdated((size as i32).to_string()));
    });
    slider_container
        .add_child(slider_title)
        .add_child(slider_value)
        .add_child(slider_widget);
    slider_container
}
fn create_control_bar(root_widget: &mut WidgetBuilder) -> (WidgetRef, WidgetRef, WidgetRef) {
    let control_color = [0.7, 0.7, 0.7, 1.0];
    let mut button_container = WidgetBuilder::new();
    let style = style!(RectStyleable::BackgroundColor: control_color);
    button_container
        .set_debug_name("button_container")
        .set_drawable_with_style(RectDrawable::new(), style)
        .hbox()
        .set_padding(30.0);
    layout!(button_container:
        match_width(root_widget),
        align_bottom(root_widget),
        shrink_vertical());
    let mut undo_widget = PushButtonBuilder::new();
    undo_widget
        .set_text("Undo")
        .set_inactive()
        .on_click(|_, _| { event!(Target::Ui, CircleEvent::Undo); });
    let mut redo_widget = PushButtonBuilder::new();
    redo_widget
        .set_text("Redo")
        .set_inactive()
        .on_click(|_, _| { event!(Target::Ui, CircleEvent::Redo); });
    let slider_container = create_slider_control();
    let (undo_id, redo_id, slider_id) = (undo_widget.widget.widget.clone(), redo_widget.widget.widget.clone(), slider_container.widget.clone());
    button_container
        .add_child(undo_widget)
        .add_child(redo_widget)
        .add_child(slider_container);
    root_widget.add_child(button_container);
    (undo_id, redo_id, slider_id)
}

fn create_circle(ui: &mut Ui, center: &Point, parent_id: WidgetRef, size: f64) -> WidgetRef {
    let style = style!(EllipseStyleable::BackgroundColor: selector!(WHITE, SELECTED: RED),
                       EllipseStyleable::Border: Some((1.0, BLACK)));
    let mut widget = WidgetBuilder::new();
    widget.set_debug_name("circle");
    widget.set_drawable_with_style(EllipseDrawable::new(), style);
    widget.add_handler(CircleHandler { center: center.clone() });
    let widget_ref = widget.widget.clone();
    widget_ref.event(ResizeEvent(size));
    widget.on_click(move |_, _| {
        event!(Target::Ui, CircleEvent::Select(Some(widget_ref.clone())));
    });
    ui.add_widget(widget, Some(parent_id))
}

struct ResizeEvent(f64);
struct CircleHandler {
    center: Point,
}
impl WidgetEventHandler<ResizeEvent> for CircleHandler {
    fn handle(&mut self, event: &ResizeEvent, mut args: WidgetEventArgs) {
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
    Resize(f64),
}

struct CircleEventHandler {
    circle_canvas_id: WidgetRef,
    undo_id: WidgetRef,
    redo_id: WidgetRef,
    slider_id: WidgetRef,

    circles: HashMap<WidgetRef, (Point, f64)>,
    undo_queue: Vec<WidgetRef>,
    redo_queue: Vec<(Point, f64)>,
    selected: Option<WidgetRef>,
}
impl CircleEventHandler {
    fn new(circle_canvas_id: WidgetRef,
           undo_id: WidgetRef,
           redo_id: WidgetRef,
           slider_id: WidgetRef)
           -> Self {
        CircleEventHandler {
            circles: HashMap::new(),
            undo_queue: Vec::new(),
            redo_queue: Vec::new(),
            circle_canvas_id: circle_canvas_id,
            undo_id: undo_id,
            redo_id: redo_id,
            slider_id: slider_id,
            selected: None,
        }
    }
}
impl UiEventHandler<CircleEvent> for CircleEventHandler {
    fn handle(&mut self, event: &CircleEvent, ui: &mut Ui) {
        match *event {
            CircleEvent::Add(point) => {
                let size = 30.0;
                let circle_id = create_circle(ui, &point, self.circle_canvas_id.clone(), size);
                self.circles.insert(circle_id.clone(), (point, size));
                self.undo_queue.push(circle_id.clone());
                self.redo_queue.clear();

                self.undo_id.event_subtree(PropChange::Remove(Property::Inactive));
                self.redo_id.event_subtree(PropChange::Add(Property::Inactive));
                event!(Target::Ui, CircleEvent::Select(Some(circle_id)));
            }
            CircleEvent::Undo => {
                if self.circles.len() > 0 {
                    let mut widget_id = self.undo_queue.pop().unwrap();
                    let (point, size) = self.circles.remove(&widget_id).unwrap();
                    widget_id.remove_widget();
                    self.redo_queue.push((point, size));

                    self.redo_id.event_subtree(PropChange::Remove(Property::Inactive));
                    if self.circles.len() == 0 {
                        self.undo_id.event_subtree(PropChange::Add(Property::Inactive));
                    }
                }
            }
            CircleEvent::Redo => {
                if self.redo_queue.len() > 0 {
                    let (point, size) = self.redo_queue.pop().unwrap();
                    let circle_id = create_circle(ui, &point, self.circle_canvas_id.clone(), size);
                    self.circles.insert(circle_id.clone(), (point, size));
                    self.undo_queue.push(circle_id);
                    if self.redo_queue.len() == 0 {
                        self.redo_id.event_subtree(PropChange::Add(Property::Inactive));
                    }
                }
            }
            CircleEvent::Select(ref new_selected) => {
                if let Some(ref selected) = self.selected {
                    selected.event_subtree(PropChange::Remove(Property::Selected));
                }
                self.selected = new_selected.clone();
                if let Some(ref selected) = self.selected {
                    let size = self.circles.get_mut(&selected).unwrap().1;
                    self.slider_id.event(SetSliderValue(size));
                    selected.event_subtree(PropChange::Add(Property::Selected));
                    self.slider_id.event_subtree(PropChange::Remove(Property::Inactive));
                } else {
                    self.slider_id.event_subtree(PropChange::Add(Property::Inactive));
                }
            }
            CircleEvent::Delete => {
                if let Some(mut selected) = self.selected.take() {
                    selected.remove_widget();
                    self.slider_id.event_subtree(PropChange::Add(Property::Inactive));
                }
            }
            CircleEvent::Resize(size) => {
                if let Some(ref selected) = self.selected {
                    selected.event(ResizeEvent(size));
                    self.circles.get_mut(&selected).unwrap().1 = size;
                }
            }
        }
    }
}

fn main() {
    let mut app = util::init_default("Limn circles demo");
    util::load_default_font();

    let mut root_widget = WidgetBuilder::new();

    let mut circle_canvas = WidgetBuilder::new();
    circle_canvas.no_container();
    layout!(circle_canvas: min_height(600.0));
    circle_canvas
        .set_debug_name("circle_canvas")
        .set_drawable_with_style(RectDrawable::new(), style!(RectStyleable::BackgroundColor: WHITE))
        .on_click(|event, _| {
            let event = CircleEvent::Add(event.position);
            event!(Target::Ui, event);
        });
    let circle_canvas_id = circle_canvas.widget.clone();
    root_widget.add_child(circle_canvas);
    let (undo_id, redo_id, slider_id) = create_control_bar(&mut root_widget);

    app.add_handler(CircleEventHandler::new(circle_canvas_id, undo_id, redo_id, slider_id));
    app.add_handler_fn(|event: &KeyboardInput, _| {
        if let &KeyboardInput(glutin::ElementState::Released, _, Some(glutin::VirtualKeyCode::Delete)) = event {
            event!(Target::Ui, CircleEvent::Delete);
        }
    });

    util::set_root_and_loop(app, root_widget);
}
