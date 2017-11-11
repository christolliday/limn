#[macro_use]
extern crate limn;

mod util;

use std::collections::HashMap;

use limn::prelude::*;

use limn::widgets::button::ButtonStyle;
use limn::widgets::edit_text::{self, EditTextBuilder, TextUpdated};
use limn::widgets::list::ListBuilder;
use limn::widgets::scroll::ScrollBuilder;
use limn::widgets::text::StaticTextStyle;
use limn::draw::text::TextComponentStyle;
use limn::draw::rect::RectState;

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
    list_widget: WidgetRef,
    first_name_box: WidgetRef,
    last_name_box: WidgetRef,
    create_button: WidgetRef,
    update_button: WidgetRef,
    delete_button: WidgetRef,
}
struct PeopleHandler {
    widgets: Widgets,
    selected_item: Option<PersonId>,
    id_gen: IdGen<PersonId>,
    person: Person,
    people: HashMap<PersonId, Person>,
    people_widgets: HashMap<PersonId, WidgetRef>,
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
            let text_style = TextComponentStyle {
                text: Some(Value::from(self.person.name())),
                text_color: Some(Value::from(WHITE)),
                ..TextComponentStyle::default()
            };
            let mut list_item_widget = WidgetBuilder::new("list_item");
            list_item_widget
                .set_style_class("list_item_rect")
                .list_item(&self.widgets.list_widget)
                .on_item_selected(move |args| {
                    args.ui.event(PeopleEvent::PersonSelected(Some(id)));
                })
                .enable_hover();
            let mut list_text_widget = WidgetBuilder::new("list_text");
            list_text_widget
                .set_draw_style(text_style)
                .add_handler(edit_text::text_change_handle);
            list_text_widget.layout().add(constraints![
                match_height(&list_item_widget),
                center(&list_item_widget)]);
            list_item_widget.add_child(list_text_widget);
            list_item_widget
        };
        self.people_widgets.insert(id, list_item_widget.widget_ref());
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
        .with_min_dimensions(100, 100);
    let mut app = util::init(window_builder);
    let mut root = WidgetBuilder::new("root");

    root.layout().add(min_size(Size::new(300.0, 300.0)));
    let mut container = WidgetBuilder::new("container");
    container.layout().add(bound_by(&root).padding(20.0));

    let create_name_group = |title, container: &mut WidgetBuilder| {
        let mut name_container = WidgetBuilder::new("name_container");
        name_container.layout().add(match_width(container));

        let mut static_text = StaticTextStyle::default();
        static_text.text(title);
        let mut static_text = WidgetBuilder::from_component_style(static_text);
        static_text.layout().add(center_vertical(&name_container));

        let mut text_box = EditTextBuilder::new();
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
    first_name_box.on_text_changed(|text, args| {
        args.ui.event(PeopleEvent::ChangeFirstName(text.0.clone()));
    });
    last_name_box.on_text_changed(|text, args| {
        args.ui.event(PeopleEvent::ChangeLastName(text.0.clone()));
    });

    let mut button_container = WidgetBuilder::new("button_container");
    button_container.layout().add(below(&last_name_container).padding(20.0));

    let mut create_button = ButtonStyle::default();
    create_button.text("Create");
    let mut create_button = WidgetBuilder::from_component_style(create_button);
    create_button.add_prop(Property::Inactive);

    let mut update_button = ButtonStyle::default();
    update_button.text("Update");
    let mut update_button = WidgetBuilder::from_component_style(update_button);
    update_button.add_prop(Property::Inactive);
    update_button.add_handler(|_: &ClickEvent, args: EventArgs| {
        args.ui.event(PeopleEvent::Update);
    });
    let mut delete_button = ButtonStyle::default();
    delete_button.text("Delete");
    let mut delete_button = WidgetBuilder::from_component_style(delete_button);
    delete_button.add_prop(Property::Inactive);
    delete_button.add_handler(|_: &ClickEvent, args: EventArgs| {
        args.ui.event(PeopleEvent::Delete);
    });
    update_button.layout().add(to_right_of(&create_button).padding(20.0));
    delete_button.layout().add(to_right_of(&update_button).padding(20.0));

    let mut scroll_container = ScrollBuilder::new();
    scroll_container.set_draw_state(RectState::default());
    scroll_container.layout().add(constraints![
        below(&button_container).padding(20.0),
        min_height(260.0),
    ]);

    let mut list_widget = ListBuilder::new();
    list_widget.on_item_selected(|selected, args| {
        if selected.is_none() {
            args.ui.event(PeopleEvent::PersonSelected(None));
        }
    });
    list_widget.layout().add(match_width(&scroll_container));

    create_button.add_handler(|_: &ClickEvent, args: EventArgs| {
        args.ui.event(PeopleEvent::Add);
    });
    let widgets = Widgets {
        list_widget: list_widget.widget_ref(),
        first_name_box: first_name_box.widget_ref(),
        last_name_box: last_name_box.widget_ref(),
        create_button: create_button.widget_ref(),
        update_button: update_button.widget_ref(),
        delete_button: delete_button.widget_ref(),
    };
    first_name_container.add_child(first_name_box);
    last_name_container.add_child(last_name_box);
    scroll_container.add_content(list_widget);
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
