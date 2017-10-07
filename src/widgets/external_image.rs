use widget::WidgetBuilder;
pub use draw::external_image::ExternalImageState;

pub struct ExternalImageBuilder;

impl ExternalImageBuilder {
    pub fn new(name: &str, texture_id: u64) -> WidgetBuilder {
        let image_draw_state = ExternalImageState::new(name, texture_id);
        let mut widget = WidgetBuilder::new("external_image");
        widget.set_draw_state(image_draw_state);
        widget
    }
}
