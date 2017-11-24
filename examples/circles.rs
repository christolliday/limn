#[macro_use]
extern crate limn;

mod util;

use std::collections::HashMap;

use limn::glutin;
use limn::prelude::*;

use limn::input::mouse::WidgetMouseButton;
use limn::widgets::button::{ButtonStyle, ToggleEvent};
use limn::widgets::slider::{SliderBuilder, SetSliderValue, SliderEvent};
use limn::widgets::drag::DragEvent;
use limn::draw::text::TextStyle;
use limn::draw::rect::{RectState, RectStyle};
use limn::draw::ellipse::{EllipseState, EllipseStyle};
use limn::widgets::edit_text::{self, TextUpdated};
use limn::widgets::text::StaticTextStyle;
use limn::input::keyboard::KeyboardInput;

struct SliderControl {
    widget: WidgetBuilder,
}

impl SliderControl {
    fn new() -> Self {
        let text_style = style!(TextStyle::TextColor: selector!(BLACK, INACTIVE: GRAY_50));
        let mut widget = WidgetBuilder::new("slider_container");
        let mut slider_title = StaticTextStyle::default();
        slider_title.style(style!(parent: text_style, TextStyle::Text: "Circle Size".to_owned()));
        let mut slider_title = WidgetBuilder::from_component(slider_title);
        slider_title.set_name("slider_title");
        slider_title.layout().add(align_left(&widget));
        let mut slider_value = StaticTextStyle::default();
        slider_value.style(style!(parent: text_style, TextStyle::Align: Align::End, TextStyle::Text: "--".to_owned()));
        let mut slider_value = WidgetBuilder::from_component(slider_value);
        slider_value
            .set_name("slider_value")
            .add_handler(edit_text::text_change_handle);
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
        slider_widget.add_handler(move |event: &SliderEvent, args: EventArgs| {
            slider_value_ref.event(TextUpdated((event.value as i32).to_string()));
            args.ui.event(AppEvent::Resize(*event));
        });
        let slider_widget_ref = slider_widget.widget_ref();
        let slider_value_ref = slider_value.widget_ref();
        widget.add_handler(move |event: &SetSliderValue, _: EventArgs| {
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
        let mut widget = WidgetBuilder::new("control_bar");
        let mut layout_settings = LinearLayoutSettings::new(Orientation::Horizontal);
        layout_settings.spacing = Spacing::Between;
        layout_settings.padding = 10.0;
        layout_settings.item_align = ItemAlignment::Center;
        widget.linear_layout(layout_settings);
        let mut create_button = ButtonStyle::default();
        create_button.text("Create Circle");
        create_button.toggle(true);
        let mut create_button = WidgetBuilder::from_component(create_button);
        create_button.add_handler(|event: &ToggleEvent, args: EventArgs| {
            match *event {
                ToggleEvent::On => {
                    args.ui.event(AppEvent::SetCreateMode(true));
                },
                ToggleEvent::Off => {
                    args.ui.event(AppEvent::SetCreateMode(false));
                }
            };
        });
        let mut undo_widget = ButtonStyle::default();
        undo_widget.text("Undo");
        let mut undo_widget = WidgetBuilder::from_component(undo_widget);
        undo_widget.add_handler(|_: &ClickEvent, args: EventArgs| {
            args.ui.event(AppEvent::Undo);
        });

        let mut redo_widget = ButtonStyle::default();
        redo_widget.text("Redo");
        let mut redo_widget = WidgetBuilder::from_component(redo_widget);
        redo_widget.add_handler(|_: &ClickEvent, args: EventArgs| {
            args.ui.event(AppEvent::Redo);
        });
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

fn create_circle(id: CircleId, circle: &Circle, parent_ref: &mut WidgetRef) -> WidgetRef {
    let style = style!(EllipseStyle::BackgroundColor: selector!(WHITE, SELECTED: RED),
                       EllipseStyle::Border: Some((2.0, BLACK)));
    let mut widget = WidgetBuilder::new("circle");
    widget
        .set_draw_state_with_style(EllipseState::new(), style)
        .make_draggable()
        .add_handler(|event: &DragEvent, args: EventArgs| {
            args.widget.event(CircleEvent::Drag(*event));
        })
        .add_handler(CircleHandler(id));
    let widget_ref = widget.widget_ref();
    let widget_ref_clone = widget.widget_ref();
    widget_ref.event(CircleEvent::Update(circle.center, circle.size));
    widget.add_handler(move |event: &WidgetMouseButton, args: EventArgs| {
        if let WidgetMouseButton(glutin::ElementState::Pressed, _) = *event {
            args.ui.event(AppEvent::Select(Some(id)));
        }
    });
    widget.add_handler(|_: &ClickEvent, args: EventArgs| *args.handled = true);
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
        self.selected = new_selected;
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
                let widget_ref = create_circle(circle_id, &circle, &mut self.circle_canvas_ref);
                self.circles.insert(circle_id, circle);
                self.circle_widgets.insert(circle_id, widget_ref);
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
                    let circle_id = self.id_gen.next_id();
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
            AppEvent::Select(new_selected) => {
                self.update_selected(new_selected);
            }
            AppEvent::Delete => {
                if let Some(selected) = self.selected.take() {
                    self.new_change(Change::Delete(selected));
                }
            }
            AppEvent::Resize(ref event) => {
                if let Some(selected) = self.selected {
                    if event.dragging {
                        let circle = &self.circles[&selected];
                        self.circle_widgets[&selected].event(CircleEvent::Update(circle.center, event.value));
                    } else {
                        self.new_change(Change::Resize(selected, event.offset));
                    }
                }
            }
            AppEvent::Move(widget_ref, change) => {
                self.new_change(Change::Move(widget_ref, change));
            }
        }
    }
}

fn main() {

    let window_builder = glutin::WindowBuilder::new()
        .with_title("Limn circles demo")
        .with_min_dimensions(100, 100);
    let mut app = util::init(window_builder);

    let mut root = WidgetBuilder::new("root");
    root.layout().add(size(Size::new(700.0, 500.0)).strength(STRONG));

    let mut circle_canvas = WidgetBuilder::new("circle_canvas");
    circle_canvas.layout().no_container();
    circle_canvas.layout().add(constraints![
        align_top(&root),
        match_width(&root),
    ]);
    circle_canvas
        .set_draw_state_with_style(RectState::new(), style!(RectStyle::BackgroundColor: WHITE))
        .add_handler(|event: &ClickEvent, args: EventArgs| {
            args.ui.event(AppEvent::ClickCanvas(event.position));
        });
    let mut control_bar = ControlBar::new();
    control_bar.layout().add(constraints![
        align_below(&circle_canvas).padding(10.0),
        align_left(&root).padding(10.0),
        align_right(&root).padding(10.0),
        align_bottom(&root).padding(10.0),
        height(100.0),
    ]);
    app.add_handler(AppEventHandler::new(circle_canvas.widget_ref(), &control_bar));
    app.add_handler(|event: &KeyboardInput, args: EventArgs| {
        if event.0.state == glutin::ElementState::Released {
            if let Some(glutin::VirtualKeyCode::Delete) = event.0.virtual_keycode {
                args.ui.event(AppEvent::Delete)
            }
        }
    });
    root.add_child(circle_canvas);
    root.add_child(control_bar);

    app.main_loop(root);
}
