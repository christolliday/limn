#[macro_use]
extern crate limn;
#[macro_use]
extern crate limn_layout;
extern crate text_layout;
extern crate cassowary;

mod util;

use std::mem;
use std::collections::HashMap;

use limn::prelude::*;

use limn::widgets::button::{PushButtonBuilder, WidgetClickable};
use limn::widgets::edit_text::{EditTextBuilder, TextUpdated};
use limn::widgets::list::{ListBuilder, STYLE_LIST_ITEM};
use limn::widgets::scroll::ScrollBuilder;
use limn::widgets::text::TextBuilder;
use limn::drawable::text::{TextDrawable, TextStyleable};
use limn::drawable::rect::RectDrawable;

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
    PersonSelected(Option<Widget>),
    ChangeFirstName(String),
    ChangeLastName(String),
}

struct Ids {
    list_widget: Widget,
    first_name_box: Widget,
    last_name_box: Widget,
    create_button: Widget,
    update_button: Widget,
    delete_button: Widget,
}
struct PeopleHandler {
    ids: Ids,
    selected_item: Option<Widget>,
    person: Person,
    people: HashMap<Widget, Person>,
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
pub fn add_person(person: &Person, mut list_widget_id: Widget) -> Widget {
    let list_item_widget = {
        let text_style = style!(TextStyleable::TextColor: WHITE);
        let text_drawable = TextDrawable::new(&person.name());
        let text_size = text_drawable.measure();
        let mut list_item_widget = Widget::new();
        list_item_widget
            .set_drawable_with_style(RectDrawable::new(), STYLE_LIST_ITEM.clone())
            .list_item(list_widget_id.clone())
            .enable_hover();
        layout!(list_item_widget: height(text_size.height));
        let mut list_text_widget = Widget::new();
        list_text_widget
            .set_drawable_with_style(text_drawable, text_style)
            .add_handler_fn(edit_text::text_change_handle);
        layout!(list_text_widget: center(&list_item_widget));
        list_item_widget.add_child(list_text_widget);
        list_item_widget
    };
    list_widget_id.add_child(list_item_widget.clone());
    list_item_widget
}

fn main() {
    let mut app = util::init_default("Limn edit text demo");
    let mut root = app.ui.root.clone();

    layout!(root: min_size(Size::new(300.0, 300.0)));
    let mut container = Widget::new();
    layout!(container: bound_by(&root).padding(20.0));

    let create_name_group = |title, container: &mut Widget| {
        let mut name_container = Widget::new();
        layout!(name_container: match_width(container));

        let mut static_text = TextBuilder::new(title);
        layout!(static_text:
            center_vertical(&name_container));

        let mut text_box = EditTextBuilder::new();
        layout!(text_box:
            min_height(30.0),
            min_width(200.0),
            align_right(&name_container),
            to_right_of(&static_text).padding(20.0));
        name_container.add_child(static_text);
        (name_container, text_box)
    };

    let (mut first_name_container, mut first_name_box) = create_name_group("First name:", &mut container);
    let (mut last_name_container, mut last_name_box) = create_name_group("Last name:", &mut container);

    layout!(first_name_container: align_top(&container));
    layout!(last_name_container: below(&first_name_container).padding(20.0));
    first_name_box.on_text_changed(|text, _| {
        event!(Target::Ui, PeopleEvent::ChangeFirstName(text.0.clone()));
    });
    last_name_box.on_text_changed(|text, _| {
        event!(Target::Ui, PeopleEvent::ChangeLastName(text.0.clone()));
    });

    let mut button_container = Widget::new();
    layout!(button_container: below(&last_name_container).padding(20.0));

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
    layout!(update_button: to_right_of(&create_button).padding(20.0));
    layout!(delete_button: to_right_of(&update_button).padding(20.0));

    let mut scroll_container = ScrollBuilder::new();
    scroll_container
        .set_drawable(RectDrawable::new());
    layout!(scroll_container:
        below(&button_container).padding(20.0),
        min_height(260.0));

    let mut list_widget = ListBuilder::new();
    list_widget.on_item_selected(|selected, _| {
        event!(Target::Ui, PeopleEvent::PersonSelected(selected));
    });
    layout!(list_widget: match_width(&scroll_container));

    create_button.on_click(|_, _| {
        event!(Target::Ui, PeopleEvent::Add);
    });
    let ids = Ids {
        list_widget: list_widget.widget.clone(),
        first_name_box: first_name_box.widget.clone(),
        last_name_box: last_name_box.widget.clone(),
        create_button: create_button.widget.clone(),
        update_button: update_button.widget.clone(),
        delete_button: delete_button.widget.clone(),
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

    app.main_loop();
}
