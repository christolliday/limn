#[macro_use]
extern crate limn;
#[macro_use]
extern crate limn_layout;

mod util;

use std::mem;
use std::collections::HashMap;

use limn::prelude::*;

use limn::widgets::button::PushButtonBuilder;
use limn::widgets::edit_text::{EditTextBuilder, TextUpdated};
use limn::widgets::list::{ListBuilder, STYLE_LIST_ITEM};
use limn::widgets::scroll::ScrollBuilder;
use limn::widgets::text::TextBuilder;
use limn::draw::text::{TextState, TextStyle};
use limn::draw::rect::RectState;

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
    PersonSelected(Option<WidgetRef>),
    ChangeFirstName(String),
    ChangeLastName(String),
}

struct Ids {
    list_widget: WidgetRef,
    first_name_box: WidgetRef,
    last_name_box: WidgetRef,
    create_button: WidgetRef,
    update_button: WidgetRef,
    delete_button: WidgetRef,
}
struct PeopleHandler {
    ids: Ids,
    selected_item: Option<WidgetRef>,
    person: Person,
    people: HashMap<WidgetRef, Person>,
}
impl PeopleHandler {
    fn new(ids: Ids) -> Self {
        PeopleHandler {
            ids: ids,
            selected_item: None,
            person: Person::new(),
            people: HashMap::new(),
        }
    }
}

impl PeopleHandler {
    fn update_selected(&mut self) {
        let ids = &self.ids;
        ids.first_name_box.event_subtree(TextUpdated(self.person.first_name.clone()));
        ids.last_name_box.event_subtree(TextUpdated(self.person.last_name.clone()));
        if self.selected_item.is_some() {
            ids.update_button.event_subtree(PropChange::Remove(Property::Inactive));
            ids.delete_button.event_subtree(PropChange::Remove(Property::Inactive));
        } else {
            ids.update_button.event_subtree(PropChange::Add(Property::Inactive));
            ids.delete_button.event_subtree(PropChange::Add(Property::Inactive));
        }
    }
}
impl UiEventHandler<PeopleEvent> for PeopleHandler {
    fn handle(&mut self, event: &PeopleEvent, _: &mut Ui) {

        let was_valid = self.person.is_valid();
        match event.clone() {
            PeopleEvent::Add => {
                if was_valid {
                    let person = mem::replace(&mut self.person, Person::new());
                    let id = add_person(&person, self.ids.list_widget.clone());
                    self.people.insert(id, person);

                    self.selected_item = None;
                    self.update_selected();
                }
            },
            PeopleEvent::Update => {
                if let Some(ref selected_widget_id) = self.selected_item {
                    self.people.insert(selected_widget_id.clone(), self.person.clone());
                    selected_widget_id.event_subtree(TextUpdated(self.person.name()));
                }
            },
            PeopleEvent::Delete => {
                if let Some(mut selected_widget_id) = self.selected_item.clone() {
                    self.people.remove(&selected_widget_id);
                    selected_widget_id.remove_widget();
                }
                self.selected_item = None;
            }
            PeopleEvent::PersonSelected(widget_id) => {
                self.selected_item = widget_id.clone();
                if let Some(widget_id) = widget_id {
                    self.person = self.people[&widget_id].clone();
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
                self.ids.create_button.event_subtree(PropChange::Remove(Property::Inactive));
            } else {
                self.ids.create_button.event_subtree(PropChange::Add(Property::Inactive));
            }
        }
    }
}

use limn::widgets::edit_text;
pub fn add_person(person: &Person, mut list_widget_id: WidgetRef) -> WidgetRef {
    let list_item_widget = {
        let text_style = style!(TextStyle::TextColor: WHITE);
        let text_draw_state = TextState::new(&person.name());
        let text_size = text_draw_state.measure();
        let mut list_item_widget = WidgetBuilder::new("list_item");
        list_item_widget
            .set_draw_state_with_style(RectState::new(), STYLE_LIST_ITEM.clone())
            .list_item(&list_widget_id)
            .enable_hover();
        list_item_widget.layout().add(constraints![
            height(text_size.height),
            match_width(&list_widget_id),
        ]);
        let mut list_text_widget = WidgetBuilder::new("list_text");
        list_text_widget
            .set_draw_state_with_style(text_draw_state, text_style)
            .add_handler_fn(edit_text::text_change_handle);
        list_text_widget.layout().add(center(&list_item_widget));
        list_item_widget.add_child(list_text_widget);
        list_item_widget
    };
    let list_item_ref = list_item_widget.widget_ref();
    list_widget_id.add_child(list_item_widget);
    list_item_ref
}

fn main() {
    let mut app = util::init_default("Limn edit text demo");
    let mut root = WidgetBuilder::new("root");

    root.layout().add(min_size(Size::new(300.0, 300.0)));
    let mut container = WidgetBuilder::new("container");
    container.layout().add(bound_by(&root).padding(20.0));

    let create_name_group = |title, container: &mut WidgetBuilder| {
        let mut name_container = WidgetBuilder::new("name_container");
        name_container.layout().add(match_width(container));

        let mut static_text = TextBuilder::new(title);
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
    first_name_box.on_text_changed(|text, _| {
        event!(Target::Ui, PeopleEvent::ChangeFirstName(text.0.clone()));
    });
    last_name_box.on_text_changed(|text, _| {
        event!(Target::Ui, PeopleEvent::ChangeLastName(text.0.clone()));
    });

    let mut button_container = WidgetBuilder::new("button_container");
    button_container.layout().add(below(&last_name_container).padding(20.0));

    let mut create_button = PushButtonBuilder::new();
    create_button.set_text("Create");
    create_button.set_inactive();
    let mut update_button = PushButtonBuilder::new();
    update_button.set_text("Update");
    update_button.set_inactive();
    update_button.on_click(|_, _| {
        event!(Target::Ui, PeopleEvent::Update);
    });
    let mut delete_button = PushButtonBuilder::new();
    delete_button.set_text("Delete");
    delete_button.set_inactive();
    delete_button.on_click(|_, _| {
        event!(Target::Ui, PeopleEvent::Delete);
    });
    update_button.layout().add(to_right_of(&create_button).padding(20.0));
    delete_button.layout().add(to_right_of(&update_button).padding(20.0));

    let mut scroll_container = ScrollBuilder::new();
    scroll_container
        .set_draw_state(RectState::new());
    scroll_container.layout().add(constraints![
        below(&button_container).padding(20.0),
        min_height(260.0),
    ]);

    let mut list_widget = ListBuilder::new();
    list_widget.on_item_selected(|selected, _| {
        event!(Target::Ui, PeopleEvent::PersonSelected(selected));
    });
    list_widget.layout().add(match_width(&scroll_container));

    create_button.on_click(|_, _| {
        event!(Target::Ui, PeopleEvent::Add);
    });
    let ids = Ids {
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

    app.add_handler(PeopleHandler::new(ids));

    app.main_loop(root);
}
