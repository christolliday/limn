use webrender::api::*;

use render::RenderBuilder;
use widget::draw::Draw;
use resources::resources;
use resources::image::ImageSource;
use geometry::{Rect, Size};
use style::Component;

#[derive(Clone, Debug)]
pub struct ImageState {
    pub image: ImageSource,
    pub scale: Size,
}

impl Component for ImageState {
    fn name() -> String {
        String::from("image")
    }
}

impl ImageState {
    pub fn new(source: ImageSource) -> Self {
        ImageState {
            image: source,
            scale: Size::new(1.0, 1.0),
        }
    }
    pub fn measure(&self) -> Size {
        let descriptor = resources().image_loader.get_image(&self.image).unwrap().descriptor;
        Size::new(descriptor.width as f32, descriptor.height as f32)
    }
    pub fn scale(&mut self, scale: Size) {
        self.scale = scale;
    }
}

impl Draw for ImageState {
    fn draw(&mut self, bounds: Rect, _: Rect, renderer: &mut RenderBuilder) {
        let key = resources().image_loader.get_image(&self.image).unwrap().key;
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
