use webrender::api::*;

use render::RenderBuilder;
use widget::draw::Draw;
use resources::resources;
use geometry::{Rect, Size};
use style::Component;

#[derive(Clone, Debug)]
pub struct ImageState {
    pub image: String,
    pub scale: Size,
}

impl Component for ImageState {
    fn name() -> String {
        String::from("image")
    }
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
        let info = PrimitiveInfo::new(bounds);
        renderer.builder.push_image(
            &info,
            bounds.size,
            LayoutSize::zero(),
            ImageRendering::Auto,
            key,
        );
    }
}
