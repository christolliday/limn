use widget::WidgetBuilder;
use draw::image::ImageState;
use layout::constraint::*;

use style::*;

#[derive(Debug, Clone)]
pub struct Image {
    file: String,
}

impl Image {
    pub fn new(file: &str) -> Self {
        Image {
            file: file.to_owned(),
        }
    }
}

impl Component for Image {
    fn name() -> String {
        String::from("image")
    }
}

impl WidgetModifier for Image {
    fn apply(&self, widget: &mut WidgetBuilder) {
        let image_draw_state = ImageState::new(&self.file);
        let image_size = image_draw_state.measure();
        widget.set_name("image");
        widget.set_draw_state(image_draw_state);
        widget.layout().add(size(image_size));
    }
}
