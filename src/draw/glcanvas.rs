use webrender::api::*;

use render::RenderBuilder;
use widget::draw::Draw;
use resources::resources;
use resources::ImageInfo;
use geometry::{Rect, RectExt, Size};
use style::Component;

#[derive(Clone, Debug)]
pub struct GLCanvasState {
    texture_id: u64,
    pub data: ImageData,
    info: ImageInfo,
}

impl Component for GLCanvasState {
    fn name() -> String {
        String::from("glcanvas")
    }
}

impl GLCanvasState {
    pub fn new(texture_id: u64) -> GLCanvasState {
        let data = ImageData::External(ExternalImageData {
            id: ExternalImageId(texture_id),
            channel_index: 0,
            image_type: ExternalImageType::Texture2DHandle,
        });
        let descriptor = ImageDescriptor::new(0, 0, ImageFormat::RGB8, true);
        let info = resources().create_texture(data.clone(), descriptor);
        GLCanvasState {
            texture_id: texture_id,
            data: data,
            info: info,
        }
    }

    pub fn measure(&self) -> Size {
        let info = self.info.info;
        Size::new(info.width as f32, info.height as f32)
    }
}

impl Draw for GLCanvasState {
    fn draw(&mut self, bounds: Rect, _: Rect, renderer: &mut RenderBuilder) {
        let mut res = resources();
        let image_info = &mut self.info;
        if bounds.width() as u32 != image_info.info.width ||
            bounds.height() as u32 != image_info.info.height {
            let descriptor = ImageDescriptor::new(bounds.width() as u32, bounds.height() as u32, ImageFormat::RGB8, true);
            let mut resources = ResourceUpdates::new();
            resources.update_image(image_info.key, descriptor, self.data.clone(), None);
            res.render_api().update_resources(resources);
            image_info.info = descriptor;
            if let ImageData::External(ExternalImageData { id: ExternalImageId(texture_id), .. }) = self.data {
                res.texture_descriptors.insert(texture_id, descriptor);
            }
        }
        let info = PrimitiveInfo::new(bounds);
        renderer.builder.push_image(
            &info,
            bounds.size,
            LayoutSize::zero(),
            ImageRendering::Auto,
            image_info.key,
        );
    }
}
