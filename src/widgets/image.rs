use widget::Widget;
use drawable::image::ImageDrawable;
use layout::constraint::*;

pub struct ImageBuilder;

impl ImageBuilder {
    pub fn new(file: &str) -> Widget {
        let image_drawable = ImageDrawable::new(file);
        let image_size = image_drawable.measure();
        let mut widget = Widget::new();
        widget.set_drawable(image_drawable);
        widget.layout().add(size(image_size));
        widget
    }
}
