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

use limn::input::mouse::WidgetMouseButton;
use limn::widgets::button::{PushButtonBuilder, ToggleButtonBuilder, ToggleEvent};
use limn::widgets::slider::{SliderBuilder, SetSliderValue, SliderEvent};
use limn::widgets::drag::DragEvent;
use limn::draw::text::TextStyle;
use limn::draw::rect::{RectState, RectStyle};
use limn::draw::ellipse::{EllipseState, EllipseStyle};
use limn::widgets::edit_text::{self, TextUpdated};
use limn::widgets::text::TextBuilder;
use limn::input::keyboard::KeyboardInput;
use limn::resources::id::{Id, IdGen};

struct SliderControl {
    widget: WidgetBuilder,
}

impl SliderControl {
    fn new() -> Self {
        let text_style = style!(TextStyle::TextColor: selector!(BLACK, INACTIVE: GRAY_50));
        let mut widget = WidgetBuilder::new("slider_container");
        let mut slider_title = TextBuilder::new_with_style(
            style!(parent: text_style, TextStyle::Text: "Circle Size".to_owned()));
        slider_title.set_name("slider_title");
        slider_title.layout().add(align_left(&widget));
        let mut slider_value = TextBuilder::new_with_style(
            style!(parent: text_style, TextStyle::Align: Align::End, TextStyle::Text: "--".to_owned()));
        slider_value
            .set_name("slider_value")
            .add_handler_fn(edit_text::text_change_handle);
        slider_value.layout().add(align_right(&widget));
        let mut slider_widget = SliderBuilder::new();
        slider_widget
            .set_range(10.0..500.0)
            .set_name("slider_widget");
        slider_widget.layout().add(constraints![
            min_width(300.0),
            below(&slider_title).padding(10.0),
            below(&slider_value).padding(10.0),
            match_width(&widget),
        ]);

        let slider_value_ref = slider_value.widget_ref();
        slider_widget.add_handler_fn(move |event: &SliderEvent, args| {
            slider_value_ref.event(TextUpdated((event.value as i32).to_string()));
            args.ui.event(AppEvent::Resize(event.clone()));
        });
        let slider_widget_ref = slider_widget.widget_ref();
        let slider_value_ref = slider_value.widget_ref();
        widget.add_handler_fn(move |event: &SetSliderValue, _| {
            let size = event.0;
            slider_widget_ref.event(SetSliderValue(size));
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
    create: WidgetRef,
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
            .hbox(30.0, true);
        let mut create_button = ToggleButtonBuilder::new();
        create_button
            .set_text("Create Circle", "Create Circle")
            .on_toggle(|event, args| {
                match *event {
                    ToggleEvent::On => {
                        args.ui.event(AppEvent::SetCreateMode(true));
                    },
                    ToggleEvent::Off => {
                        args.ui.event(AppEvent::SetCreateMode(false));
                    }
                };
            });
        create_button.layout().add(center_vertical(&widget));
        let mut undo_widget = PushButtonBuilder::new();
        undo_widget
            .set_text("Undo")
            .on_click(|_, args| { args.ui.event(AppEvent::Undo); });
        undo_widget.layout().add(center_vertical(&widget));
        let mut redo_widget = PushButtonBuilder::new();
        redo_widget
            .set_text("Redo")
            .on_click(|_, args| { args.ui.event(AppEvent::Redo); });
        redo_widget.layout().add(center_vertical(&widget));
        let slider_container = SliderControl::new();
        let (create_ref, undo_ref, redo_ref, slider_ref) = (create_button.widget_ref(), undo_widget.widget_ref(), redo_widget.widget_ref(), slider_container.widget_ref());
        widget
            .add_child(create_button)
            .add_child(undo_widget)
            .add_child(redo_widget)
            .add_child(slider_container);
        ControlBar {
            widget: widget,
            create: create_ref,
            undo: undo_ref,
            redo: redo_ref,
            slider: slider_ref,
        }
    }
}
widget_wrapper!(ControlBar);

fn create_circle(id: CircleId, circle: Circle, mut parent_ref: WidgetRef) -> WidgetRef {
    let style = style!(EllipseStyle::BackgroundColor: selector!(WHITE, SELECTED: RED),
                       EllipseStyle::Border: Some((2.0, BLACK)));
    let mut widget = WidgetBuilder::new("circle");
    widget
        .set_draw_state_with_style(EllipseState::new(), style)
        .make_draggable()
        .add_handler_fn(|event: &DragEvent, args| {
            args.widget.event(CircleEvent::Drag(event.clone()));
        })
        .add_handler(CircleHandler(id));
    let widget_ref = widget.widget_ref();
    let widget_ref_clone = widget.widget_ref();
    widget_ref.event(CircleEvent::Update(circle.center, circle.size));
    widget.add_handler_fn(move |event: &WidgetMouseButton, args| {
        if let &WidgetMouseButton(glutin::ElementState::Pressed, _) = event {
            args.ui.event(AppEvent::Select(Some(id)));
        }
    });
    widget.on_click(|_, args| *args.handled = true);
    parent_ref.add_child(widget);
    widget_ref_clone
}

enum CircleEvent {
    Update(Point, f32),
    Drag(DragEvent),
}
struct CircleHandler(CircleId);
impl EventHandler<CircleEvent> for CircleHandler {
    fn handle(&mut self, event: &CircleEvent, args: EventArgs) {
        match *event {
            CircleEvent::Update(center, size) => {
                args.widget.update_layout(|layout| {
                    layout.edit_top().set(center.y - size / 2.0);
                    layout.edit_left().set(center.x - size / 2.0);
                    layout.edit_width().set(size);
                    layout.edit_height().set(size);
                });
            }
            CircleEvent::Drag(ref event) => {
                args.ui.event(AppEvent::Move(self.0, event.change));
            }
        }
    }
}

enum Change {
    Create(CircleId, Circle),
    Delete(CircleId),
    Resize(CircleId, f32),
    Move(CircleId, Vector),
    None,
}

enum AppEvent {
    SetCreateMode(bool),
    ClickCanvas(Point),
    Undo,
    Redo,
    Select(Option<CircleId>),
    Delete,
    Resize(SliderEvent),
    Move(CircleId, Vector),
}

#[derive(Clone)]
struct Circle {
    center: Point,
    size: f32,
}

named_id!(CircleId);

struct AppEventHandler {
    circle_canvas_ref: WidgetRef,
    create_ref: WidgetRef,
    undo_ref: WidgetRef,
    redo_ref: WidgetRef,
    slider_ref: WidgetRef,

    id_gen: IdGen<CircleId>,
    circle_widgets: HashMap<CircleId, WidgetRef>,
    create_mode: bool,
    circles: HashMap<CircleId, Circle>,
    undo_queue: Vec<Change>,
    redo_queue: Vec<Change>,
    selected: Option<CircleId>,
}

impl AppEventHandler {
    fn new(circle_canvas_ref: WidgetRef, control_bar: &ControlBar) -> Self {
        let mut handler = AppEventHandler {
            circle_canvas_ref: circle_canvas_ref,
            create_ref: control_bar.create.clone(),
            undo_ref: control_bar.undo.clone(),
            redo_ref: control_bar.redo.clone(),
            slider_ref: control_bar.slider.clone(),

            id_gen: IdGen::new(),
            circle_widgets: HashMap::new(),
            create_mode: true,
            circles: HashMap::new(),
            undo_queue: Vec::new(),
            redo_queue: Vec::new(),
            selected: None,
        };
        handler.create_ref.add_prop(Property::Activated);
        handler.undo_ref.add_prop(Property::Inactive);
        handler.redo_ref.add_prop(Property::Inactive);
        handler.slider_ref.add_prop(Property::Inactive);
        handler
    }
    fn update_selected(&mut self, new_selected: Option<CircleId>) {
        if let Some(ref selected) = self.selected {
            self.circle_widgets.get_mut(selected).unwrap().remove_prop(Property::Selected);
        }
        self.selected = new_selected.clone();
        if let Some(ref selected) = self.selected {
            let size = self.circles[selected].size;
            self.slider_ref.event(SetSliderValue(size));
            self.circle_widgets.get_mut(selected).unwrap().add_prop(Property::Selected);
            self.slider_ref.remove_prop(Property::Inactive);
        } else {
            self.slider_ref.add_prop(Property::Inactive);
        }
    }
    fn apply_change(&mut self, change: Change) -> Change {
        match change {
            Change::Create(circle_id, circle) => {
                self.circles.insert(circle_id, circle.clone());
                let widget_ref = create_circle(circle_id, circle, self.circle_canvas_ref.clone());
                self.circle_widgets.insert(circle_id, widget_ref.clone());
                self.update_selected(Some(circle_id));
                Change::Delete(circle_id)
            },
            Change::Delete(circle_id) => {
                self.update_selected(None);
                let mut widget_ref = self.circle_widgets.remove(&circle_id).unwrap();
                widget_ref.remove_widget();
                let circle = self.circles.remove(&circle_id).unwrap();
                Change::Create(circle_id, circle)
            }
            Change::Resize(circle_id, size_change) => {
                let circle = self.circles.get_mut(&circle_id).unwrap();
                circle.size += size_change;
                self.circle_widgets[&circle_id].event(CircleEvent::Update(circle.center, circle.size));
                self.slider_ref.event(SetSliderValue(circle.size));
                Change::Resize(circle_id, -size_change)
            }
            Change::Move(circle_id, pos_change) => {
                let circle = self.circles.get_mut(&circle_id).unwrap();
                circle.center += pos_change;
                self.circle_widgets[&circle_id].event(CircleEvent::Update(circle.center, circle.size));
                Change::Move(circle_id, -pos_change)
            }
            _ => Change::None
        }
    }
    fn new_change(&mut self, change: Change) {
        let change = self.apply_change(change);
        self.undo_queue.push(change);
        self.redo_queue.clear();
        self.undo_ref.remove_prop(Property::Inactive);
        self.redo_ref.add_prop(Property::Inactive);
    }
    fn undo(&mut self) {
        if let Some(change) = self.undo_queue.pop() {
            let change = self.apply_change(change);
            self.redo_queue.push(change);
            self.redo_ref.remove_prop(Property::Inactive);
            if self.undo_queue.is_empty() {
                self.undo_ref.add_prop(Property::Inactive);
            }
        }
    }
    fn redo(&mut self) {
        if let Some(change) = self.redo_queue.pop() {
            let change = self.apply_change(change);
            self.undo_queue.push(change);
            self.undo_ref.remove_prop(Property::Inactive);
            if self.redo_queue.is_empty() {
                self.redo_ref.add_prop(Property::Inactive);
            }
        }
    }
}
impl EventHandler<AppEvent> for AppEventHandler {
    fn handle(&mut self, event: &AppEvent, _: EventArgs) {
        match *event {
            AppEvent::SetCreateMode(on) => {
                self.create_mode = on;
            }
            AppEvent::ClickCanvas(point) => {
                if self.create_mode {
                    let circle = Circle { center: point, size: 100.0 };
                    let circle_id = self.id_gen.next();
                    self.new_change(Change::Create(circle_id, circle));
                } else {
                    self.update_selected(None);
                }
            }
            AppEvent::Undo => {
                self.undo();
            }
            AppEvent::Redo => {
                self.redo();
            }
            AppEvent::Select(ref new_selected) => {
                self.update_selected(new_selected.clone());
            }
            AppEvent::Delete => {
                if let Some(selected) = self.selected.take() {
                    self.new_change(Change::Delete(selected));
                }
            }
            AppEvent::Resize(ref event) => {
                if let Some(selected) = self.selected.clone() {
                    if event.dragging {
                        let circle = &self.circles[&selected];
                        self.circle_widgets[&selected].event(CircleEvent::Update(circle.center, event.value));
                    } else {
                        self.new_change(Change::Resize(selected, event.offset));
                    }
                }
            }
            AppEvent::Move(ref widget_ref, change) => {
                self.new_change(Change::Move(widget_ref.clone(), change));
            }
        }
    }
}

fn main() {
    let mut app = util::init_default("Limn circles demo");
    let mut root = WidgetBuilder::new("root");
    root.vbox(0.0, false);

    let mut circle_canvas = WidgetBuilder::new("circle_canvas");
    circle_canvas.no_container();
    circle_canvas.layout().add(constraints![
        match_width(&root),
        min_height(600.0),
    ]);
    circle_canvas
        .set_draw_state_with_style(RectState::new(), style!(RectStyle::BackgroundColor: WHITE))
        .on_click(|event, args| {
            args.ui.event(AppEvent::ClickCanvas(event.position));
        });
    let mut control_bar = ControlBar::new();
    control_bar.layout().add(constraints![
        match_width(&root),
        align_bottom(&root),
        shrink_vertical(),
    ]);
    app.add_handler(AppEventHandler::new(circle_canvas.widget_ref(), &control_bar));
    app.add_handler_fn(|event: &KeyboardInput, args| {
        if let KeyboardInput(glutin::ElementState::Released, _, Some(glutin::VirtualKeyCode::Delete)) = *event {
            args.ui.event(AppEvent::Delete);
        }
    });
    root.add_child(circle_canvas);
    root.add_child(control_bar);
    app.main_loop(root);
}
