#[allow(unused_imports)]
#[macro_use]
extern crate limn;
extern crate lipsum;
extern crate rand;

use lipsum::lipsum;
use rand::Rng;

mod util;

use limn::prelude::*;
use limn::widgets::list;

fn main() {
    let window_builder = glutin::WindowBuilder::new()
        .with_title("Limn list demo")
        .with_min_dimensions(glutin::dpi::LogicalSize{width: 100.0, height: 300.0});
    let app = util::init(window_builder);
    let mut root = Widget::new("root");

    let mut list_widget = Widget::from_modifier(List::default());
    let list_data = (0..15).map(|_| {
        let rand = rand::thread_rng().gen_range(1, 6);
        lipsum(rand)
    });
    list::add_contents_to_list(&mut list_widget, list_data, list::default_text_adapter);

    let mut scroll_widget = ScrollContainer::default();
    scroll_widget.add_content(list_widget.clone());
    let mut scroll_widget = Widget::from_modifier(scroll_widget);
    list_widget.layout().add(constraints![shrink(), match_width(&scroll_widget)]);
    scroll_widget.layout().add(constraints![
        bound_by(&root).padding(50.0),
        size(Size::new(300.0, 300.0)),
    ]);
    root.add_child(scroll_widget);

    app.main_loop(root);
}
