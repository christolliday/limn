use graphics;
use super::super::util::*;
use super::super::ui::Resources;
use backend::gfx::G2d;
use backend::gfx::ImageSize;
use graphics::Context;
use graphics::Transformed;
use resources::Id;
use std::any::Any;

pub struct ImageDrawable {
    pub image_id: Id,
    pub scale: Dimensions,
}
impl ImageDrawable {
    pub fn new(image_id: Id) -> Self {
        ImageDrawable { image_id: image_id, scale: Dimensions { width: 1.0, height: 1.0 } }
    }
    pub fn measure_image(&self, resources: &Resources) -> Dimensions {
        let img = resources.images.get(self.image_id).unwrap();
        img.get_size().into()
    }
    pub fn scale(&mut self, scale: Dimensions) {
        self.scale = scale;
    }
}

pub fn draw_image(state: &Any,
                  parent_bounds: Rectangle,
                  bounds: Rectangle,
                  resources: &mut Resources,
                  context: Context,
                  graphics: &mut G2d) {
    let state: &ImageDrawable = state.downcast_ref().unwrap();
    //let context = crop_context(context, parent_bounds);

    let img = resources.images.get(state.image_id).unwrap();
    let dims: Dimensions = img.get_size().into();
    let scale = bounds.dims() / dims;
    let image = graphics::image::Image::new();
    image.rect(bounds);
    let mut context = context.trans(bounds.left, bounds.top).scale(scale.width, scale.height);

    image.draw(img, &context.draw_state, context.transform, graphics);
}
