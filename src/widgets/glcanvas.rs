use widget::WidgetBuilder;
pub use draw::glcanvas::GLCanvasState;

pub struct GLCanvasBuilder;

impl GLCanvasBuilder {
    pub fn new(name: &str, texture_id: u64) -> WidgetBuilder {
        let image_draw_state = GLCanvasState::new(name, texture_id);
        let mut widget = WidgetBuilder::new("glcanvas");
        widget.set_draw_state(image_draw_state);
        widget
    }
}
