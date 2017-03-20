extern crate limn;
extern crate text_layout;
extern crate cassowary;

use cassowary::WeightedRelation::*;
use cassowary::strength::*;

mod util;

use limn::event::Target;
use limn::widget::{WidgetBuilder, EventHandler, EventArgs};
use limn::widget::style::Value;
use limn::widgets::button::PushButtonBuilder;
use limn::widgets::edit_text::EditTextBuilder;
use limn::widgets::list::{STYLE_LIST_ITEM, ListItemHandler, ListHandler};
use limn::drawable::text::{TextDrawable, TextStyleField};
use limn::drawable::rect::RectDrawable;
use limn::resources::WidgetId;
use limn::ui;
use limn::ui::Ui;
use limn::ui::graph::WidgetGraph;
use limn::ui::solver::LimnSolver;
use limn::util::Dimensions;
use limn::color::*;

#[derive(Clone)]
pub struct Person {
    first_name: String,
    last_name: String,
}
impl Person {
    fn new(first_name: &str, last_name: &str) -> Self {
        Person {
            first_name: first_name.to_owned(),
            last_name: last_name.to_owned(),
        }
    }
    fn name(&self) -> String {
        format!("{}, {}", self.last_name, self.first_name)
    }
}

#[derive(Clone)]
enum PeopleEvent {
    Add,
    Update,
    Delete,
    ChangeFirstName(String),
    ChangeLastName(String),
}

struct PeopleHandler {
    list_widget_id: WidgetId,
    people: Vec<Person>,
    person: Person,
}
impl PeopleHandler {
    fn new(list_widget_id: WidgetId) -> Self {
        PeopleHandler {
            list_widget_id: list_widget_id,
            people: Vec::new(),
            person: Person::new("", ""),
        }
    }
}
impl ui::EventHandler<PeopleEvent> for PeopleHandler {
    fn handle(&mut self, event: &PeopleEvent, args: ui::EventArgs) {
        match event.clone() {
            PeopleEvent::Add => {
                let person = self.person.clone();
                add_person(&person, &mut args.ui.graph, self.list_widget_id, &mut args.ui.solver);
                self.people.push(person);
            },
            PeopleEvent::ChangeFirstName(name) => {
                self.person.first_name = name;
            },
            PeopleEvent::ChangeLastName(name) => {
                self.person.last_name = name;
            }, _ => ()
        }
    }
}

pub fn add_person(person: &Person, graph: &mut WidgetGraph, list_widget_id: WidgetId, solver: &mut LimnSolver) {
    let list_item_widget = {
        
        let list_widget = graph.get_widget(list_widget_id).unwrap();
        let text_style = vec![TextStyleField::TextColor(Value::Single(WHITE))];
        let text_drawable = TextDrawable::new(&person.name());
        let text_dims = text_drawable.measure();
        let mut list_item_widget = WidgetBuilder::new();
        list_item_widget
            .set_drawable_with_style(RectDrawable::new(), STYLE_LIST_ITEM.clone())
            .set_debug_name("item")
            .add_handler(ListItemHandler::new(list_widget_id))
            .enable_hover();
        list_item_widget.layout.match_width(&list_widget.layout);
        list_item_widget.layout.height(text_dims.height);
        let mut list_text_widget = WidgetBuilder::new();
        list_text_widget
            .set_drawable_with_style(text_drawable, text_style)
            .set_debug_name("text");
        list_text_widget.layout.center(&list_item_widget.layout.vars);
        list_item_widget.add_child(list_text_widget);
        list_item_widget
    };
    graph.add_widget(list_item_widget, Some(list_widget_id), solver);
}

fn main() {
    let (window, mut app) = util::init_default("Limn edit text demo");
    util::load_default_font();

    let mut root_widget = WidgetBuilder::new();
    root_widget.layout.min_dimensions(Dimensions {
        width: 300.0,
        height: 300.0,
    });
    let mut container = WidgetBuilder::new();
    container.layout.bound_by(&root_widget.layout.vars).padding(20.0);

    let mut first_name_container = WidgetBuilder::new();
    first_name_container.layout.align_top(&container.layout.vars);

    let mut first_name = WidgetBuilder::new();
    let text = TextDrawable::new("First name:");
    let text_dims = text.measure();
    first_name.set_drawable(text);
    first_name.layout.center_vertical(&first_name_container.layout.vars);
    first_name.layout.dimensions(text_dims);

    let mut first_name_box = EditTextBuilder::new();
    first_name_box.on_text_changed(|text, args| {
        args.queue.push(Target::Ui, PeopleEvent::ChangeFirstName(text.0.clone()));
    });
    let mut first_name_box = first_name_box.widget;
    first_name_box.set_debug_name("first_name");
    first_name_box.layout.min_height(30.0);
    first_name_box.layout.min_width(200.0);
    first_name_box.layout.align_right(&container.layout.vars);
    first_name_box.layout.to_right_of(&first_name.layout.vars).padding(20.0);
    first_name_container.layout.shrink();
    first_name_container.add_child(first_name);
    first_name_container.add_child(first_name_box);

    let mut last_name_container = WidgetBuilder::new();
    last_name_container.layout.below(&first_name_container.layout.vars).padding(20.0);
    let mut last_name = WidgetBuilder::new();
    let text = TextDrawable::new("Last name:");
    let text_dims = text.measure();
    last_name.set_drawable(text);
    last_name.layout.dimensions(text_dims);

    let mut last_name_box = EditTextBuilder::new();
    last_name_box.on_text_changed(|text, args| {
        args.queue.push(Target::Ui, PeopleEvent::ChangeLastName(text.0.clone()));
    });
    let mut last_name_box = last_name_box.widget;
    last_name_box.set_debug_name("last_name");
    last_name_box.layout.min_height(30.0);
    last_name_box.layout.align_right(&container.layout.vars);
    last_name_box.layout.to_right_of(&last_name.layout.vars).padding(20.0);
    last_name_container.add_child(last_name);
    last_name_container.add_child(last_name_box);

    let mut button_container = WidgetBuilder::new();
    button_container.layout.below(&last_name_container.layout.vars).padding(20.0);

    let mut create_button = PushButtonBuilder::new();
    create_button.set_text("Create");
    let mut create_button = create_button.widget;
    let mut update_button = PushButtonBuilder::new();
    update_button.set_text("Update");
    let mut update_button = update_button.widget;
    update_button.on_click(|_, _| {
        println!("update");
    });
    let mut delete_button = PushButtonBuilder::new();
    delete_button.set_text("Delete");
    let mut delete_button = delete_button.widget;
    delete_button.on_click(|_, _| {
        println!("delete");
    });
    update_button.layout.to_right_of(&create_button.layout.vars).padding(20.0);
    delete_button.layout.to_right_of(&update_button.layout.vars).padding(20.0);

    let mut scroll_container = WidgetBuilder::new();
    scroll_container.set_drawable(RectDrawable::new());
    scroll_container.layout.below(&button_container.layout.vars).padding(20.0);
    scroll_container.layout.min_height(260.0);
    scroll_container.contents_scroll();

    let mut list_widget = WidgetBuilder::new();
    list_widget
        .add_handler(ListHandler::new())
        .vbox()
        .make_scrollable();
    list_widget.layout.match_width(&scroll_container.layout.vars);
    let list_widget_id = list_widget.id;
    scroll_container.add_child(list_widget);

    create_button.on_click(move |_, args| {
        let event = PeopleEvent::Add;
        args.queue.push(Target::Ui, event);
        println!("create");
    });
    button_container.add_child(create_button);
    button_container.add_child(update_button);
    button_container.add_child(delete_button);

    container.add_child(first_name_container);
    container.add_child(last_name_container);
    container.add_child(button_container);
    container.add_child(scroll_container);
    root_widget.add_child(container);

    app.add_handler(PeopleHandler::new(list_widget_id));

    util::set_root_and_loop(window, app, root_widget);
}
