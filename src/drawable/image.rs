use webrender_api::*;

use render::RenderBuilder;
use widget::drawable::Drawable;
use resources::resources;
use util::{self, Rect, RectExt, Size, SizeExt};

pub struct ImageDrawable {
    pub image: String,
    pub scale: Size,
}
impl ImageDrawable {
    pub fn new(image: &str) -> Self {
        ImageDrawable {
            image: image.to_owned(),
            scale: Size::new(1.0, 1.0),
        }
    }
    pub fn measure(&self) -> Size {
        let mut res = resources();
        let info = res.get_image(&self.image).info;
        Size::new(info.width as f32, info.height as f32)
    }
    pub fn scale(&mut self, scale: Size) {
        self.scale = scale;
    }
}
impl Drawable for ImageDrawable {
    fn draw(&mut self, bounds: Rect, _: Rect, renderer: &mut RenderBuilder) {
        let key = resources().get_image(&self.image).key;
        let rect = util::to_layout_rect(bounds);
        renderer.builder.push_image(
            rect,
            None,
            rect.size,
            LayoutSize::zero(),
            ImageRendering::Auto,
            key,
        );
    }
}
