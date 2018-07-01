use webrender::api::*;

use render::RenderBuilder;
use widget::draw::Draw;
use resources::resources;
use resources::image::ImageInfo;
use geometry::{Rect, RectExt, Size};
use style::Component;

#[derive(Clone, Debug)]
pub struct GLCanvasState {
    data: ExternalImageData,
    image_info: ImageInfo,
}

impl Component for GLCanvasState {
    fn name() -> String {
        String::from("glcanvas")
    }
}

impl GLCanvasState {
    pub fn new(texture_id: u64) -> GLCanvasState {
        let data = ExternalImageData {
            id: ExternalImageId(texture_id),
            channel_index: 0,
            image_type: ExternalImageType::TextureHandle(TextureTarget::Default),
        };
        let descriptor = ImageDescriptor::new(0, 0, ImageFormat::BGRA8, true, false);
        let image_info = resources().image_loader.create_image_resource(ImageData::External(data), descriptor);
        GLCanvasState {
            data: data,
            image_info: image_info,
        }
    }

    pub fn measure(&self) -> Size {
        let descriptor = self.image_info.descriptor;
        Size::new(descriptor.size.width as f32, descriptor.size.height as f32)
    }
}

impl Draw for GLCanvasState {
    fn draw(&mut self, bounds: Rect, _: Rect, renderer: &mut RenderBuilder) {
        let descriptor = self.image_info.descriptor;
        let (bounds_width, bounds_height) = (bounds.width() as u32, bounds.height() as u32);
        if bounds_width != descriptor.size.width || bounds_height != descriptor.size.height {
            let descriptor = ImageDescriptor::new(bounds_width, bounds_height, ImageFormat::BGRA8, true, false);
            resources().image_loader.update_texture(self.image_info.key, descriptor, self.data);
            self.image_info.descriptor = descriptor;
        }
        renderer.builder.push_image(
            &PrimitiveInfo::new(bounds),
            bounds.size,
            LayoutSize::zero(),
            ImageRendering::Auto,
            AlphaType::Alpha,
            self.image_info.key,
        );
    }
}
