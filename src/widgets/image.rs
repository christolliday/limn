use widget::WidgetBuilder;
use draw::image::ImageState;
use layout::constraint::*;

#[derive(Debug, Copy, Clone)]
pub struct ImageBuilder;

impl ImageBuilder {
    #[cfg_attr(feature = "cargo-clippy", allow(new_ret_no_self))]
    pub fn new(file: &str) -> WidgetBuilder {
        let image_draw_state = ImageState::new(file);
        let image_size = image_draw_state.measure();
        let mut widget = WidgetBuilder::new("image");
        widget.set_draw_state(image_draw_state);
        widget.layout().add(size(image_size));
        widget
    }
}
