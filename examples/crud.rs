extern crate limn;
extern crate text_layout;
extern crate cassowary;

mod util;

use std::mem;

use limn::event::Target;
use limn::widget::{WidgetBuilder, EventHandler, EventArgs};
use limn::widget::WidgetBuilderCore;
use limn::widget::style::Value;
use limn::widget::property::PropChange;
use limn::widgets::button::{PushButtonBuilder, WidgetClickable};
use limn::widgets::edit_text::{EditTextBuilder, TextUpdated};
use limn::widgets::list::STYLE_LIST_ITEM;
use limn::widgets::linear_layout::LinearLayoutEvent;
use limn::drawable::text::{TextDrawable, TextStyleField};
use limn::drawable::rect::RectDrawable;
use limn::resources::WidgetId;
use limn::ui::{self, Ui};
use limn::event::Queue;
use limn::util::Dimensions;
use limn::color::*;

#[derive(Clone, Debug)]
pub struct Person {
    first_name: String,
    last_name: String,
}
impl Person {
    fn new() -> Self {
        Person {
            first_name: String::new(),
            last_name: String::new(),
        }
    }
    fn name(&self) -> String {
        format!("{}, {}", self.last_name, self.first_name)
    }
    fn is_valid(&self) -> bool {
        self.first_name.len() > 0 && self.last_name.len() > 0
    }
}

#[derive(Clone)]
enum PeopleEvent {
    Add,
    Update,
    Delete,
    PersonSelected(usize, WidgetId),
    ChangeFirstName(String),
    ChangeLastName(String),
}

struct PeopleHandler {
    list_widget_id: WidgetId,
    first_name_box_id: WidgetId,
    last_name_box_id: WidgetId,
    create_button_id: WidgetId,
    update_button_id: WidgetId,
    delete_button_id: WidgetId,
    selected_item: Option<(usize, WidgetId)>,
    person: Person,
    people: Vec<Person>,
}
impl PeopleHandler {
    fn new(list_widget_id: WidgetId,
           first_name_box_id: WidgetId,
           last_name_box_id: WidgetId,
           create_button_id: WidgetId,
           update_button_id: WidgetId,
           delete_button_id: WidgetId
    ) -> Self {
        PeopleHandler {
            list_widget_id: list_widget_id,
            first_name_box_id: first_name_box_id,
            last_name_box_id: last_name_box_id,
            create_button_id: create_button_id,
            update_button_id: update_button_id,
            delete_button_id: delete_button_id,
            selected_item: None,
            person: Person::new(),
            people: Vec::new(),
        }
    }
}

impl PeopleHandler {
    fn update_selected(&mut self, queue: &mut Queue) {
        queue.push(Target::SubTree(self.first_name_box_id), TextUpdated(self.person.first_name.clone()));
        queue.push(Target::SubTree(self.last_name_box_id), TextUpdated(self.person.last_name.clone()));
        if self.selected_item.is_some() {
            queue.push(Target::SubTree(self.update_button_id), PropChange::Remove(Property::Inactive));
            queue.push(Target::SubTree(self.delete_button_id), PropChange::Remove(Property::Inactive));
        } else {
            queue.push(Target::SubTree(self.update_button_id), PropChange::Add(Property::Inactive));
            queue.push(Target::SubTree(self.delete_button_id), PropChange::Add(Property::Inactive));
        }
    }
}
impl ui::EventHandler<PeopleEvent> for PeopleHandler {
    fn handle(&mut self, event: &PeopleEvent, args: ui::EventArgs) {

        let was_valid = self.person.is_valid();
        match event.clone() {
            PeopleEvent::Add => {
                if was_valid {
                    let person = mem::replace(&mut self.person, Person::new());
                    add_person(&person, self.people.len(), args.ui, self.list_widget_id);
                    self.people.push(person);

                    self.selected_item = None;
                    self.update_selected(args.queue);
                }
            },
            PeopleEvent::Update => {
                if let Some((selected_person_index, selected_widget)) = self.selected_item {
                    self.people[selected_person_index] = self.person.clone();
                    args.queue.push(Target::SubTree(selected_widget), TextUpdated(self.person.name()));
                }
            },
            PeopleEvent::Delete => {
                if let Some((selected_person_index, selected_widget)) = self.selected_item {
                    self.people.remove(selected_person_index);
                    let event = LinearLayoutEvent::RemoveWidget(selected_widget);
                    args.queue.push(Target::Widget(self.list_widget_id), event);
                    args.ui.remove_widget(selected_widget);
                }
            }
            PeopleEvent::PersonSelected(person_index, widget_id) => {
                self.person = self.people[person_index].clone();
                self.selected_item = Some((person_index, widget_id));
                self.update_selected(args.queue);
            },
            PeopleEvent::ChangeFirstName(name) => {
                self.person.first_name = name;
            },
            PeopleEvent::ChangeLastName(name) => {
                self.person.last_name = name;
            }
        }
        let is_valid = self.person.is_valid();
        if was_valid != is_valid {
            if is_valid {
                args.queue.push(Target::SubTree(self.create_button_id), PropChange::Remove(Property::Inactive));
            } else {
                args.queue.push(Target::SubTree(self.create_button_id), PropChange::Add(Property::Inactive));
            }
        }
    }
}

struct PersonHandler {
    person_index: usize,
}
impl PersonHandler {
    fn new(person_index: usize) -> Self {
        PersonHandler {
            person_index: person_index,
        }
    }
}
use limn::widget::property::Property;
impl EventHandler<PropChange> for PersonHandler {
    fn handle(&mut self, event: &PropChange, args: EventArgs) {
        match *event {
            PropChange::Add(Property::Selected) => {
                args.queue.push(Target::Ui, PeopleEvent::PersonSelected(self.person_index, args.widget.id));
            },
            PropChange::Remove(Property::Selected) => {
                //println!("{:?}", event);
            }, _ => ()
        }
    }
}

use limn::widgets::edit_text::TextChangeHandler;
pub fn add_person(person: &Person, person_index: usize, ui: &mut Ui, list_widget_id: WidgetId) {
    let list_item_widget = {
        let text_style = vec![TextStyleField::TextColor(Value::Single(WHITE))];
        let text_drawable = TextDrawable::new(&person.name());
        let text_dims = text_drawable.measure();
        let mut list_item_widget = WidgetBuilder::new();
        list_item_widget
            .set_drawable_with_style(RectDrawable::new(), STYLE_LIST_ITEM.clone())
            .add_handler(PersonHandler::new(person_index))
            .list_item(list_widget_id)
            .enable_hover();
        list_item_widget.layout().height(text_dims.height);
        let mut list_text_widget = WidgetBuilder::new();
        list_text_widget
            .set_drawable_with_style(text_drawable, text_style)
            .add_handler(TextChangeHandler);
        list_text_widget.layout().center(&list_item_widget.layout());
        list_item_widget.add_child(list_text_widget);
        list_item_widget
    };
    ui.add_widget(list_item_widget, Some(list_widget_id));
}

fn main() {
    let (window, mut app) = util::init_default("Limn edit text demo");
    util::load_default_font();

    let mut root_widget = WidgetBuilder::new();
    root_widget.layout().min_dimensions(Dimensions {
        width: 300.0,
        height: 300.0,
    });
    let mut container = WidgetBuilder::new();
    container.layout().bound_by(&root_widget.layout()).padding(20.0);

    let create_name_group = |title, container: &mut WidgetBuilder| {
        let mut name_container = WidgetBuilder::new();
        name_container.layout().match_width(container.layout());

        let mut static_text = WidgetBuilder::new();
        let text = TextDrawable::new(title);
        let text_dims = text.measure();
        static_text.set_drawable(text);
        static_text.layout().center_vertical(&name_container.layout());
        static_text.layout().dimensions(text_dims);

        let mut text_box = EditTextBuilder::new();
        text_box.layout().min_height(30.0);
        text_box.layout().min_width(200.0);
        text_box.layout().align_right(&name_container.layout());
        text_box.layout().to_right_of(&static_text.layout()).padding(20.0);
        name_container.add_child(static_text);
        (name_container, text_box)
    };

    let (mut first_name_container, mut first_name_box) = create_name_group("First name:", &mut container);
    let (mut last_name_container, mut last_name_box) = create_name_group("Last name:", &mut container);

    first_name_container.layout().align_top(&container.layout());
    last_name_container.layout().below(&first_name_container.layout()).padding(20.0);
    first_name_box.on_text_changed(|text, args| {
        args.queue.push(Target::Ui, PeopleEvent::ChangeFirstName(text.0.clone()));
    });
    last_name_box.on_text_changed(|text, args| {
        args.queue.push(Target::Ui, PeopleEvent::ChangeLastName(text.0.clone()));
    });
    let first_name_box_id = first_name_box.id();
    let last_name_box_id = last_name_box.id();
    first_name_container.add_child(first_name_box.widget);
    last_name_container.add_child(last_name_box.widget);

    let mut button_container = WidgetBuilder::new();
    button_container.layout().below(&last_name_container.layout()).padding(20.0);

    let mut create_button = PushButtonBuilder::new();
    create_button.set_text("Create");
    create_button.set_inactive();
    let mut update_button = PushButtonBuilder::new();
    update_button.set_text("Update");
    update_button.set_inactive();
    update_button.on_click(|_, args| {
        args.queue.push(Target::Ui, PeopleEvent::Update);
    });
    let mut delete_button = PushButtonBuilder::new();
    delete_button.set_text("Delete");
    delete_button.set_inactive();
    delete_button.on_click(|_, args| {
        args.queue.push(Target::Ui, PeopleEvent::Delete);
    });
    update_button.layout().to_right_of(&create_button.layout()).padding(20.0);
    delete_button.layout().to_right_of(&update_button.layout()).padding(20.0);

    let mut scroll_container = WidgetBuilder::new();
    scroll_container.set_drawable(RectDrawable::new());
    scroll_container.layout().below(&button_container.layout()).padding(20.0);
    scroll_container.layout().min_height(260.0);
    scroll_container.contents_scroll();

    let mut list_widget = WidgetBuilder::new();
    list_widget.make_vertical_list();
    list_widget.layout().match_width(&scroll_container.layout());
    let list_widget_id = list_widget.id();
    scroll_container.add_child(list_widget);

    create_button.on_click(move |_, args| {
        args.queue.push(Target::Ui, PeopleEvent::Add);
    });
    let create_button_id = create_button.id();
    let update_button_id = update_button.id();
    let delete_button_id = delete_button.id();
    button_container.add_child(create_button.widget);
    button_container.add_child(update_button.widget);
    button_container.add_child(delete_button.widget);

    container.add_child(first_name_container);
    container.add_child(last_name_container);
    container.add_child(button_container);
    container.add_child(scroll_container);
    root_widget.add_child(container);

    app.add_handler(PeopleHandler::new(list_widget_id, first_name_box_id, last_name_box_id, create_button_id, update_button_id, delete_button_id));

    util::set_root_and_loop(window, app, root_widget);
}
