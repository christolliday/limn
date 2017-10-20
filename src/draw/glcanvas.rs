use webrender::api::*;

use render::RenderBuilder;
use widget::draw::Draw;
use resources::get_global_resources;
use geometry::{Rect, RectExt, Size, SizeExt};

pub struct GLCanvasState {
    pub name: String,
    pub data: ImageData,
}

impl GLCanvasState {
    pub fn new(name: &str, texture_id: u64) -> GLCanvasState {
        let data = ImageData::External(ExternalImageData {
            id: ExternalImageId(texture_id),
            channel_index: 0,
            image_type: ExternalImageType::Texture2DHandle,
        });
        let descriptor = ImageDescriptor::new(0, 0, ImageFormat::RGB8, true);
        get_global_resources().put_image(name, data.clone(), descriptor);
        GLCanvasState {
            name: name.to_owned(),
            data: data,
        }
    }

    pub fn measure(&self) -> Size {
        let mut res = get_global_resources();
        let info = res.get_image(&self.name).info;
        Size::new(info.width as f32, info.height as f32)
    }
}

impl Draw for GLCanvasState {
    fn draw(&mut self, bounds: Rect, _: Rect, renderer: &mut RenderBuilder) {
        let mut res = get_global_resources();
        let image_info = res.get_image(&self.name).clone();
        if bounds.width() as u32 != image_info.info.width ||
            bounds.height() as u32 != image_info.info.height {
            let descriptor = ImageDescriptor::new(bounds.width() as u32, bounds.height() as u32, ImageFormat::RGB8, true);
            res.update_image(&self.name, self.data.clone(), descriptor);
            if let ImageData::External(ExternalImageData { id: ExternalImageId(texture_id), .. }) = self.data {
                res.texture_descriptors.insert(texture_id, descriptor);
            }
        }
        let info = PrimitiveInfo::new(bounds.typed());
        renderer.builder.push_image(
            &info,
            bounds.size.typed(),
            LayoutSize::zero(),
            ImageRendering::Auto,
            image_info.key,
        );
    }
}
