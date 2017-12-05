use widget::Widget;
use draw::image::ImageState;
use layout::constraint::*;
use resources::image::ImageSource;

use style::*;

#[derive(Debug, Clone)]
pub struct Image {
    source: ImageSource,
}

impl Image {
    pub fn new(source: ImageSource) -> Self {
        Image {
            source: source,
        }
    }
}

impl Component for Image {
    fn name() -> String {
        String::from("image")
    }
}

impl WidgetModifier for Image {
    fn apply(&self, widget: &mut Widget) {
        let image_draw_state = ImageState::new(self.source.clone());
        let image_size = image_draw_state.measure();
        widget.set_name("image");
        widget.set_draw_state(image_draw_state);
        widget.layout().add(size(image_size));
    }
}
