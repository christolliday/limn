extern crate limn;
extern crate glutin;
extern crate cassowary;

mod util;

use limn::widget::WidgetBuilder;
use limn::widgets::slider::SliderBuilder;
use limn::util::Dimensions;

fn main() {
    let (window, ui) = util::init_default("Limn slider demo");
    util::load_default_font();

    let mut root_widget = WidgetBuilder::new();
    root_widget.layout.dimensions(Dimensions {
        width: 300.0,
        height: 300.0,
    });

    let mut slider = SliderBuilder::new();
    slider.on_val_changed(|val| {
        println!("val {}", val);
    });
    let mut slider = slider.widget;
    slider.layout.align_top(&root_widget, Some(10.0));
    slider.layout.center_horizontal(&root_widget);

    root_widget.add_child(slider);

    util::set_root_and_loop(window, ui, root_widget);
}
