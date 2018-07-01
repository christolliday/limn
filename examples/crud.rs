#[allow(unused_imports)]
#[macro_use]
extern crate limn;

mod util;

use std::collections::HashMap;

use limn::prelude::*;

named_id!(PersonId);

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
        !self.first_name.is_empty() && !self.last_name.is_empty()
    }
}

#[derive(Clone)]
enum PeopleEvent {
    Add,
    Update,
    Delete,
    PersonSelected(Option<PersonId>),
    ChangeFirstName(String),
    ChangeLastName(String),
}

struct Widgets {
    list_widget: Widget,
    first_name_box: Widget,
    last_name_box: Widget,
    create_button: Widget,
    update_button: Widget,
    delete_button: Widget,
}
struct PeopleHandler {
    widgets: Widgets,
    selected_item: Option<PersonId>,
    id_gen: IdGen<PersonId>,
    person: Person,
    people: HashMap<PersonId, Person>,
    people_widgets: HashMap<PersonId, Widget>,
}
impl PeopleHandler {
    fn new(widgets: Widgets) -> Self {
        PeopleHandler {
            widgets: widgets,
            selected_item: None,
            id_gen: IdGen::new(),
            person: Person::new(),
            people: HashMap::new(),
            people_widgets: HashMap::new(),
        }
    }
}

impl PeopleHandler {
    fn update_selected(&mut self) {
        let widgets = &mut self.widgets;
        widgets.first_name_box.event_subtree(TextUpdated(self.person.first_name.clone()));
        widgets.last_name_box.event_subtree(TextUpdated(self.person.last_name.clone()));
        if self.selected_item.is_some() {
            widgets.update_button.remove_prop(Property::Inactive);
            widgets.delete_button.remove_prop(Property::Inactive);
        } else {
            widgets.update_button.add_prop(Property::Inactive);
            widgets.delete_button.add_prop(Property::Inactive);
        }
    }
    fn add_person(&mut self) {
        let id = self.id_gen.next_id();
        self.people.insert(id, self.person.clone());
        let list_item_widget = {
            let text_style = style!(TextStyle {
                text: self.person.name(),
            });
            let text_style = StaticTextStyle {
                style: Some(text_style),
            };
            let mut list_item_widget = Widget::new("list_item");
            list_item_widget
                .set_draw_style(DrawStyle::from_class::<RectStyle>("list_item_rect"))
                .add_handler(ListItemHandler::new(self.widgets.list_widget.clone()))
                .add_handler(move |_: &ItemSelected, args: EventArgs| {
                    args.ui.event(PeopleEvent::PersonSelected(Some(id)));
                })
                .enable_hover();

            let mut list_text_widget = Widget::from_modifier_style(text_style);
            list_text_widget
                .set_draw_style(DrawStyle::from_class::<RectStyle>("list_item_text"));
            list_text_widget.layout().add(constraints![
                match_height(&list_item_widget),
                align_left(&list_item_widget)]);
            list_item_widget.add_child(list_text_widget);
            list_item_widget
        };
        self.people_widgets.insert(id, list_item_widget.clone());
        self.widgets.list_widget.add_child(list_item_widget);
    }
}
impl EventHandler<PeopleEvent> for PeopleHandler {
    fn handle(&mut self, event: &PeopleEvent, _: EventArgs) {

        let was_valid = self.person.is_valid();
        match event.clone() {
            PeopleEvent::Add => {
                if was_valid {
                    self.add_person();
                    self.selected_item = None;
                    self.update_selected();
                }
            },
            PeopleEvent::Update => {
                if let Some(selected_id) = self.selected_item {
                    self.people.insert(selected_id, self.person.clone());
                    self.people_widgets[&selected_id].event_subtree(TextUpdated(self.person.name()));
                }
            },
            PeopleEvent::Delete => {
                if let Some(selected_id) = self.selected_item {
                    self.people.remove(&selected_id);
                    let mut widget = self.people_widgets.remove(&selected_id).unwrap();
                    widget.remove_widget();
                }
                self.selected_item = None;
            }
            PeopleEvent::PersonSelected(person_id) => {
                self.selected_item = person_id;
                if let Some(person_id) = person_id {
                    self.person = self.people[&person_id].clone();
                } else {
                    self.person = Person::new();
                }
                self.update_selected();
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
                self.widgets.create_button.remove_prop(Property::Inactive);
            } else {
                self.widgets.create_button.add_prop(Property::Inactive);
            }
        }
    }
}

fn main() {
    let window_builder = glutin::WindowBuilder::new()
        .with_title("Limn CRUD demo")
        .with_min_dimensions(glutin::dpi::LogicalSize{width: 100.0, height: 100.0});
    let mut app = util::init(window_builder);
    let mut root = Widget::new("root");

    root.layout().add(min_size(Size::new(300.0, 300.0)));
    let mut container = Widget::new("container");
    container.layout().add(bound_by(&root).padding(20.0));

    let create_name_group = |title, container: &mut Widget| {
        let mut name_container = Widget::new("name_container");
        name_container.layout().add(match_width(container));

        let mut static_text = Widget::from_modifier_style(StaticTextStyle::from_text(title));
        static_text.layout().add(center_vertical(&name_container));

        let mut text_box = Widget::from_modifier(EditText::default());
        text_box.layout().add(constraints![
            min_height(30.0),
            min_width(200.0),
            align_right(&name_container),
            to_right_of(&static_text).padding(20.0),
        ]);
        name_container.add_child(static_text);
        (name_container, text_box)
    };

    let (mut first_name_container, mut first_name_box) = create_name_group("First name:", &mut container);
    let (mut last_name_container, mut last_name_box) = create_name_group("Last name:", &mut container);

    first_name_container.layout().add(align_top(&container));
    last_name_container.layout().add(below(&first_name_container).padding(20.0));
    first_name_box.add_handler(|event: &TextUpdated, args: EventArgs| {
        args.ui.event(PeopleEvent::ChangeFirstName(event.0.clone()));
    });
    last_name_box.add_handler(|event: &TextUpdated, args: EventArgs| {
        args.ui.event(PeopleEvent::ChangeLastName(event.0.clone()));
    });

    let mut button_container = Widget::new("button_container");
    button_container.layout().add(below(&last_name_container).padding(20.0));

    let mut create_button = Widget::from_modifier_style(ButtonStyle::from_text("Create"));
    create_button.add_prop(Property::Inactive);

    let mut update_button = Widget::from_modifier_style(ButtonStyle::from_text("Update"));
    update_button.add_prop(Property::Inactive);
    update_button.add_handler(|_: &ClickEvent, args: EventArgs| {
        args.ui.event(PeopleEvent::Update);
    });
    let mut delete_button = Widget::from_modifier_style(ButtonStyle::from_text("Delete"));
    delete_button.add_prop(Property::Inactive);
    delete_button.add_handler(|_: &ClickEvent, args: EventArgs| {
        args.ui.event(PeopleEvent::Delete);
    });
    update_button.layout().add(to_right_of(&create_button).padding(20.0));
    delete_button.layout().add(to_right_of(&update_button).padding(20.0));

    let mut scroll_container = ScrollContainer::default();
    let mut list_widget = Widget::from_modifier(List::default());
    scroll_container.add_content(list_widget.clone());
    let mut scroll_container = Widget::from_modifier(scroll_container);
    scroll_container.set_draw_state(RectState::default());
    scroll_container.layout().add(constraints![
        below(&button_container).padding(20.0),
        match_width(&container),
        min_height(260.0),
    ]);

    list_widget.add_handler(move |event: &ListItemSelected, args: EventArgs| {
        if event.widget.is_none() {
            args.ui.event(PeopleEvent::PersonSelected(None));
        }
    });
    list_widget.layout().add(match_width(&scroll_container));

    create_button.add_handler(|_: &ClickEvent, args: EventArgs| {
        args.ui.event(PeopleEvent::Add);
    });
    let widgets = Widgets {
        list_widget: list_widget,
        first_name_box: first_name_box.clone(),
        last_name_box: last_name_box.clone(),
        create_button: create_button.clone(),
        update_button: update_button.clone(),
        delete_button: delete_button.clone(),
    };
    first_name_container.add_child(first_name_box);
    last_name_container.add_child(last_name_box);
    button_container
        .add_child(create_button)
        .add_child(update_button)
        .add_child(delete_button);

    container
        .add_child(first_name_container)
        .add_child(last_name_container)
        .add_child(button_container)
        .add_child(scroll_container);
    root.add_child(container);

    app.add_handler(PeopleHandler::new(widgets));

    app.main_loop(root);
}
