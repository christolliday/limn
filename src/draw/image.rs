use webrender_api::*;

use render::RenderBuilder;
use widget::draw::Draw;
use resources::resources;
use util::{Rect, RectExt, Size, SizeExt};

pub struct ImageState {
    pub image: String,
    pub scale: Size,
}
impl ImageState {
    pub fn new(image: &str) -> Self {
        ImageState {
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
impl Draw for ImageState {
    fn draw(&mut self, bounds: Rect, _: Rect, renderer: &mut RenderBuilder) {
        let key = resources().get_image(&self.image).key;
        renderer.builder.push_image(
            bounds.typed(),
            None,
            bounds.size.typed(),
            LayoutSize::zero(),
            ImageRendering::Auto,
            key,
        );
    }
}
