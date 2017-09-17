use widget::WidgetBuilder;
use drawable::image::ImageDrawable;
use layout::constraint::*;

pub struct ImageBuilder;

impl ImageBuilder {
    pub fn new(file: &str) -> WidgetBuilder {
        let image_drawable = ImageDrawable::new(file);
        let image_size = image_drawable.measure();
        let mut widget = WidgetBuilder::new("image");
        widget.set_drawable(image_drawable);
        widget.layout().add(size(image_size));
        widget
    }
}
