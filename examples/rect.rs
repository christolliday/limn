extern crate limn;

mod util;

use limn::prelude::*;
use limn::draw::rect::{RectComponent, RectComponentStyle};

fn main() {
    let window_builder = glutin::WindowBuilder::new()
        .with_title("Limn image demo")
        .with_min_dimensions(100, 100);
    let app = util::init(window_builder);
    let mut root = WidgetBuilder::new("root");

    let mut rect = WidgetBuilder::new("rect");
    rect.set_draw_style(RectComponentStyle::default());
    /* rect.set_draw_component(RectComponent {
        background_color: Value::from(GREEN),
        corner_radius: Value::from(None),
        border: Value::from(None),
    }); */
    rect.layout().add(constraints![
        size(Size::new(100.0, 100.0)),
        center(&root),
        bound_by(&root).padding(50.0),
    ]);
    root.add_child(rect);

    app.main_loop(root);
}
