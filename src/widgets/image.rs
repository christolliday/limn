use widget::WidgetBuilder;
use draw::image::ImageState;
use layout::constraint::*;

use style::*;

#[derive(Debug)]
pub struct ImageComponent {
    file: String,
}

impl ImageComponent {
    pub fn new(file: &str) -> Self {
        ImageComponent {
            file: file.to_owned(),
        }
    }
}

impl Component for ImageComponent {
    fn name() -> String {
        "image".to_owned()
    }
    fn apply(&self, widget: &mut WidgetBuilder) {
        let image_draw_state = ImageState::new(&self.file);
        let image_size = image_draw_state.measure();
        widget.set_name("image");
        widget.set_draw_state(image_draw_state);
        widget.layout().add(size(image_size));
    }
}
