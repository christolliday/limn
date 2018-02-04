#[allow(unused_imports)]
#[macro_use]
extern crate limn;

mod util;

use std::collections::HashMap;
use std::ops::Range;

use limn::glutin;
use limn::prelude::*;

use limn::input::mouse::WidgetMouseButton;
use limn::widgets::button::{ButtonStyle, ToggleButtonStyle, ToggleEvent};
use limn::widgets::slider::{Slider, SetSliderValue, SliderEvent};
use limn::draw::text::TextStyle;
use limn::draw::rect::{RectStyle};
use limn::draw::ellipse::{self, EllipseStyle};
use limn::widgets::edit_text::TextUpdated;
use limn::widgets::text::StaticTextStyle;
use limn::input::keyboard::KeyboardInput;
use limn::input::drag::DragEvent;


fn create_slider_control<F: FnMut(&SliderEvent, EventArgs) + 'static>(title: &str, range: Range<f32>, mut on_slider_event: F) -> Widget {
    let mut widget = Widget::new("slider_container");
    let slider_title = StaticTextStyle {
        style: Some(style!(TextStyle {
            text: String::from(title),
        }))
    };
    let mut slider_title = Widget::from_modifier_style(slider_title);
    slider_title
        .set_draw_style(DrawStyle::from_class::<TextStyle>("static_text"))
        .set_name("slider_title");
    slider_title.layout().add(align_left(&widget));
    let slider_value = StaticTextStyle {
        style: Some(style!(TextStyle {
            align: Align::End,
            text: String::from("--"),
        }))
    };
    let mut slider_value = Widget::from_modifier_style(slider_value);
    slider_value
        .set_draw_style(DrawStyle::from_class::<TextStyle>("static_text"))
        .set_name("slider_value");
    slider_value.layout().add(align_right(&widget));
    let mut slider_widget = Slider::default();
    slider_widget.set_range(range);
    let mut slider_widget = Widget::from_modifier(slider_widget);
    slider_widget.layout().add(constraints![
        min_width(300.0),
        below(&slider_title).padding(10.0),
        below(&slider_value).padding(10.0),
        match_width(&widget),
    ]);
    let slider_value_ref = slider_value.clone();
    slider_widget.add_handler(move |event: &SliderEvent, args: EventArgs| {
        slider_value_ref.event(TextUpdated(event.value.to_string()));
        on_slider_event(event, args);
    });
    let slider_widget_ref = slider_widget.clone();
    let slider_value_ref = slider_value.clone();
    widget.add_handler(move |event: &SetSliderValue, _: EventArgs| {
        let size = event.0;
        slider_widget_ref.event(SetSliderValue(size));
        slider_value_ref.event(TextUpdated(size.to_string()));
    });
    widget
        .add_child(slider_title)
        .add_child(slider_value)
        .add_child(slider_widget);
    widget
}

struct ControlBarRefs {
    create: Widget,
    undo: Widget,
    redo: Widget,
    size_slider: Widget,
    alpha_slider: Widget,
}

fn create_control_bar() -> (Widget, ControlBarRefs) {
    let mut widget = Widget::new("control_bar");
    let mut layout_settings = LinearLayoutSettings::new(Orientation::Horizontal);
    layout_settings.spacing = Spacing::Between;
    layout_settings.padding = 10.0;
    layout_settings.item_align = ItemAlignment::Center;
    widget.linear_layout(layout_settings);
    let mut create_button = ToggleButtonStyle::default();
    create_button.text("Create Circle");
    let mut create_button = Widget::from_modifier_style(create_button);
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
    let mut undo_widget = Widget::from_modifier_style(ButtonStyle::from_text("Undo"));
    undo_widget.add_handler(|_: &ClickEvent, args: EventArgs| {
        args.ui.event(AppEvent::Undo);
    });

    let mut redo_widget = Widget::from_modifier_style(ButtonStyle::from_text("Redo"));
    redo_widget.add_handler(|_: &ClickEvent, args: EventArgs| {
        args.ui.event(AppEvent::Redo);
    });
    let size_slider = create_slider_control("Circle size", 10.0..500.0, |event, args| args.ui.event(AppEvent::Resize(*event)));
    let alpha_slider = create_slider_control("Circle opacity", 0.0..1.0, |event, args| args.ui.event(AppEvent::ChangeOpacity(*event)));
    let (create_ref, undo_ref, redo_ref, size_slider_ref, alpha_slider_ref) = (create_button.clone(),
        undo_widget.clone(), redo_widget.clone(),
        size_slider.clone(), alpha_slider.clone());
    widget
        .add_child(create_button)
        .add_child(undo_widget)
        .add_child(redo_widget)
        .add_child(size_slider)
        .add_child(alpha_slider);

    (widget, ControlBarRefs {
        create: create_ref,
        undo: undo_ref,
        redo: redo_ref,
        size_slider: size_slider_ref,
        alpha_slider: alpha_slider_ref,
    })
}

fn create_circle(id: CircleId, circle: &Circle, parent_ref: &mut Widget) -> Widget {

    let mut draw_style = DrawStyle::from(style!(EllipseStyle {
        background_color: WHITE,
        border: Some((2.0, BLACK)),
    }));
    draw_style.prop_style(SELECTED.clone(), style!(EllipseStyle {
        background_color: RED,
    }));
    let mut widget = Widget::new("circle");
    widget
        .set_draw_style(draw_style)
        .add_filter(OpacityFilter::default())
        .set_cursor_hit_fn(ellipse::cursor_hit)
        .make_draggable()
        .add_handler(|event: &DragEvent, args: EventArgs| {
            args.widget.event(CircleEvent::Drag(*event));
        })
        .add_handler(CircleHandler(id));
    let widget_ref = widget.clone();
    let widget_ref_clone = widget.clone();
    widget_ref.event(CircleEvent::Update(circle.center, circle.size, circle.alpha));
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
    Update(Point, f32, f32),
    Drag(DragEvent),
}
struct CircleHandler(CircleId);
impl EventHandler<CircleEvent> for CircleHandler {
    fn handle(&mut self, event: &CircleEvent, mut args: EventArgs) {
        match *event {
            CircleEvent::Update(center, size, alpha) => {
                args.widget.update_filter(|filter: &mut OpacityFilter| {
                    filter.alpha = alpha;
                });
                let mut layout = args.widget.layout();
                layout.edit_top().set(center.y - size / 2.0);
                layout.edit_left().set(center.x - size / 2.0);
                layout.edit_width().set(size);
                layout.edit_height().set(size);
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
    ChangeOpacity(CircleId, f32),
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
    ChangeOpacity(SliderEvent),
    Move(CircleId, Vector),
}

#[derive(Clone)]
struct Circle {
    center: Point,
    size: f32,
    alpha: f32,
}

named_id!(CircleId);

struct AppEventHandler {
    circle_canvas_ref: Widget,
    create_ref: Widget,
    undo_ref: Widget,
    redo_ref: Widget,
    size_slider_ref: Widget,
    alpha_slider_ref: Widget,

    id_gen: IdGen<CircleId>,
    circle_widgets: HashMap<CircleId, Widget>,
    create_mode: bool,
    circles: HashMap<CircleId, Circle>,
    undo_queue: Vec<Change>,
    redo_queue: Vec<Change>,
    selected: Option<CircleId>,
}

impl AppEventHandler {
    fn new(circle_canvas_ref: Widget, control_bar: &ControlBarRefs) -> Self {
        let mut handler = AppEventHandler {
            circle_canvas_ref: circle_canvas_ref,
            create_ref: control_bar.create.clone(),
            undo_ref: control_bar.undo.clone(),
            redo_ref: control_bar.redo.clone(),
            size_slider_ref: control_bar.size_slider.clone(),
            alpha_slider_ref: control_bar.alpha_slider.clone(),

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
        handler.size_slider_ref.add_prop(Property::Inactive);
        handler.alpha_slider_ref.add_prop(Property::Inactive);
        handler
    }
    fn update_selected(&mut self, new_selected: Option<CircleId>) {
        if let Some(ref selected) = self.selected {
            self.circle_widgets.get_mut(selected).unwrap().remove_prop(Property::Selected);
        }
        self.selected = new_selected;
        if let Some(ref selected) = self.selected {
            let size = self.circles[selected].size;
            let alpha = self.circles[selected].alpha;
            self.size_slider_ref.event(SetSliderValue(size));
            self.alpha_slider_ref.event(SetSliderValue(alpha));
            self.circle_widgets.get_mut(selected).unwrap().add_prop(Property::Selected);
            self.size_slider_ref.remove_prop(Property::Inactive);
            self.alpha_slider_ref.remove_prop(Property::Inactive);
        } else {
            self.size_slider_ref.add_prop(Property::Inactive);
            self.alpha_slider_ref.add_prop(Property::Inactive);
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
                self.circle_widgets[&circle_id].event(CircleEvent::Update(circle.center, circle.size, circle.alpha));
                self.size_slider_ref.event(SetSliderValue(circle.size));
                Change::Resize(circle_id, -size_change)
            }
            Change::ChangeOpacity(circle_id, alpha_change) => {
                let circle = self.circles.get_mut(&circle_id).unwrap();
                circle.alpha += alpha_change;
                self.circle_widgets[&circle_id].event(CircleEvent::Update(circle.center, circle.size, circle.alpha));
                self.alpha_slider_ref.event(SetSliderValue(circle.alpha));
                Change::ChangeOpacity(circle_id, -alpha_change)
            }
            Change::Move(circle_id, pos_change) => {
                let circle = self.circles.get_mut(&circle_id).unwrap();
                circle.center += pos_change;
                self.circle_widgets[&circle_id].event(CircleEvent::Update(circle.center, circle.size, circle.alpha));
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
                    let circle = Circle { center: point, size: 100.0, alpha: 1.0 };
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
                        self.circle_widgets[&selected].event(CircleEvent::Update(circle.center, event.value, circle.alpha));
                    } else {
                        self.new_change(Change::Resize(selected, event.offset));
                    }
                }
            }
            AppEvent::ChangeOpacity(ref event) => {
                if let Some(selected) = self.selected {
                    if event.dragging {
                        let circle = &self.circles[&selected];
                        self.circle_widgets[&selected].event(CircleEvent::Update(circle.center, circle.size, event.value));
                    } else {
                        self.new_change(Change::ChangeOpacity(selected, event.offset));
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

    let mut root = Widget::new("root");
    root.layout().add(size(Size::new(700.0, 500.0)).strength(STRONG));

    let mut circle_canvas = Widget::new("circle_canvas");
    circle_canvas.layout().no_container();
    circle_canvas.layout().add(constraints![
        align_top(&root),
        match_width(&root),
    ]);
    circle_canvas
        .set_draw_style(style!(RectStyle {
            background_color: WHITE,
        }))
        .add_handler(|event: &ClickEvent, args: EventArgs| {
            args.ui.event(AppEvent::ClickCanvas(event.position));
        });
    let (mut control_bar, control_bar_refs) = create_control_bar();
    control_bar.layout().add(constraints![
        align_below(&circle_canvas).padding(10.0),
        align_left(&root).padding(10.0),
        align_right(&root).padding(10.0),
        align_bottom(&root).padding(10.0),
        height(100.0),
    ]);
    app.add_handler(AppEventHandler::new(circle_canvas.clone(), &control_bar_refs));
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
