#[macro_use]
extern crate limn;
#[macro_use]
extern crate limn_layout;
extern crate text_layout;
extern crate glutin;
extern crate cassowary;

mod util;

use std::collections::HashMap;

use text_layout::Align;

use limn::prelude::*;

use limn::widgets::button::{PushButtonBuilder, WidgetClickable};
use limn::widgets::slider::{SliderBuilder, SetSliderValue};
use limn::drawable::text::TextStyleable;
use limn::drawable::rect::{RectDrawable, RectStyleable};
use limn::drawable::ellipse::{EllipseDrawable, EllipseStyleable};
use limn::widgets::edit_text::{self, TextUpdated};
use limn::widgets::text::TextBuilder;
use limn::input::keyboard::KeyboardInput;

fn create_slider_control() -> Widget {

    let text_style = style!(TextStyleable::TextColor: selector!(BLACK, INACTIVE: GRAY_50));
    let mut slider_container = Widget::new();
    slider_container
        .set_debug_name("slider_container")
        .set_inactive();
    let mut slider_title = TextBuilder::new_with_style(
        style!(parent: text_style, TextStyleable::Text: "Circle Size".to_owned()));
    slider_title.set_debug_name("slider_title");
    layout!(slider_title: align_left(&slider_container));
    let mut slider_value = TextBuilder::new_with_style(
        style!(parent: text_style, TextStyleable::Align: Align::End, TextStyleable::Text: "30".to_owned()));
    slider_value
        .set_debug_name("slider_value")
        .add_handler_fn(edit_text::text_change_handle);
    layout!(slider_value:
        align_right(&slider_container));
    let mut slider_widget = SliderBuilder::new();
    slider_widget.set_debug_name("slider_widget");
    layout!(slider_widget:
        min_width(300.0),
        below(&slider_title).padding(10.0),
        below(&slider_value).padding(10.0),
        match_width(&slider_container));

    let slider_value_ref = slider_value.clone();
    slider_widget.on_value_changed(move |size, _| {
        let size = size * 100.0;
        slider_value_ref.event(TextUpdated((size as i32).to_string()));
        event!(Target::Ui, CircleEvent::Resize(size));
    }).set_value(0.30);
    let slider_widget = slider_widget.build();
    let slider_ref = slider_widget.clone();
    let slider_value_ref = slider_value.clone();
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
fn create_control_bar(root_widget: &mut Widget) -> (Widget, Widget, Widget) {
    let control_color = GRAY_70;
    let mut button_container = Widget::new();
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
    layout!(undo_widget:
        center_vertical(&button_container));
    let mut redo_widget = PushButtonBuilder::new();
    redo_widget
        .set_text("Redo")
        .set_inactive()
        .on_click(|_, _| { event!(Target::Ui, CircleEvent::Redo); });
    layout!(redo_widget:
        center_vertical(&button_container));
    let slider_container = create_slider_control().build();
    let (undo_widget, redo_widget) = (undo_widget.build(), redo_widget.build());
    button_container
        .add_child(undo_widget.clone())
        .add_child(redo_widget.clone())
        .add_child(slider_container.clone());
    root_widget.add_child(button_container);
    (undo_widget, redo_widget, slider_container)
}

fn create_circle(center: &Point, mut parent_id: Widget, size: f32) -> Widget {
    let style = style!(EllipseStyleable::BackgroundColor: selector!(WHITE, SELECTED: RED),
                       EllipseStyleable::Border: Some((1.0, BLACK)));
    let mut widget = Widget::new();
    widget.set_debug_name("circle");
    widget.set_drawable_with_style(EllipseDrawable::new(), style);
    widget.add_handler(CircleHandler { center: *center });
    let widget_ref = widget.clone();
    widget_ref.event(ResizeEvent(size));
    widget.on_click(move |_, _| {
        event!(Target::Ui, CircleEvent::Select(Some(widget_ref.clone())));
    });
    parent_id.add_child(widget.clone());
    widget
}

struct ResizeEvent(f32);
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
    Select(Option<Widget>),
    Delete,
    Resize(f32),
}

struct CircleEventHandler {
    circle_canvas_id: Widget,
    undo_id: Widget,
    redo_id: Widget,
    slider_id: Widget,

    circles: HashMap<Widget, (Point, f32)>,
    undo_queue: Vec<Widget>,
    redo_queue: Vec<(Point, f32)>,
    selected: Option<Widget>,
}
impl CircleEventHandler {
    fn new(circle_canvas_id: Widget,
           undo_id: Widget,
           redo_id: Widget,
           slider_id: Widget)
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
    fn handle(&mut self, event: &CircleEvent, _: &mut Ui) {
        match *event {
            CircleEvent::Add(point) => {
                let size = 30.0;
                let circle_id = create_circle(&point, self.circle_canvas_id.clone(), size);
                self.circles.insert(circle_id.clone(), (point, size));
                self.undo_queue.push(circle_id.clone());
                self.redo_queue.clear();

                self.undo_id.event_subtree(PropChange::Remove(Property::Inactive));
                self.redo_id.event_subtree(PropChange::Add(Property::Inactive));
                event!(Target::Ui, CircleEvent::Select(Some(circle_id)));
            }
            CircleEvent::Undo => {
                if !self.circles.is_empty() {
                    let mut widget_id = self.undo_queue.pop().unwrap();
                    let (point, size) = self.circles.remove(&widget_id).unwrap();
                    widget_id.remove_widget();
                    self.redo_queue.push((point, size));

                    self.redo_id.event_subtree(PropChange::Remove(Property::Inactive));
                    if self.circles.is_empty() {
                        self.undo_id.event_subtree(PropChange::Add(Property::Inactive));
                    }
                }
            }
            CircleEvent::Redo => {
                if !self.redo_queue.is_empty() {
                    let (point, size) = self.redo_queue.pop().unwrap();
                    let circle_id = create_circle(&point, self.circle_canvas_id.clone(), size);
                    self.circles.insert(circle_id.clone(), (point, size));
                    self.undo_queue.push(circle_id);
                    if self.redo_queue.is_empty() {
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
                    let size = self.circles.get_mut(selected).unwrap().1;
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
                    self.circles.get_mut(selected).unwrap().1 = size;
                }
            }
        }
    }
}

fn main() {
    let mut app = util::init_default("Limn circles demo");

    let mut root_widget = Widget::new();

    let mut circle_canvas = Widget::new();
    circle_canvas.no_container();
    layout!(circle_canvas:
        min_height(600.0));
    circle_canvas
        .set_debug_name("circle_canvas")
        .set_drawable_with_style(RectDrawable::new(), style!(RectStyleable::BackgroundColor: WHITE))
        .on_click(|event, _| {
            let event = CircleEvent::Add(event.position);
            event!(Target::Ui, event);
        });
    let circle_canvas_id = circle_canvas.clone();
    root_widget.add_child(circle_canvas);
    let (undo_id, redo_id, slider_id) = create_control_bar(&mut root_widget);

    app.add_handler(CircleEventHandler::new(circle_canvas_id, undo_id, redo_id, slider_id));
    app.add_handler_fn(|event: &KeyboardInput, _| {
        if let KeyboardInput(glutin::ElementState::Released, _, Some(glutin::VirtualKeyCode::Delete)) = *event {
            event!(Target::Ui, CircleEvent::Delete);
        }
    });

    util::set_root_and_loop(app, root_widget);
}
