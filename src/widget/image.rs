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
}
impl ImageDrawable {
    pub fn measure_image(&self, resources: &Resources) -> Dimensions {
        let img = resources.images.get(self.image_id).unwrap();
        img.get_size().into()
    }
}

pub fn draw_image(state: &Any,
                  bounds: Rectangle,
                  resources: &mut Resources,
                  context: Context,
                  graphics: &mut G2d) {
    let state: &ImageDrawable = state.downcast_ref().unwrap();

    let img = resources.images.get(state.image_id).unwrap();
    let dims: Dimensions = img.get_size().into();
    //let scale = bounds.dims() / dims;
    let scale = Dimensions { width: 2.0, height: 2.0 };
    let image = graphics::image::Image::new();
    image.rect(bounds);
    let context = context.trans(bounds.left, bounds.top).scale(scale.width, scale.height);
    image.draw(img, &context.draw_state, context.transform, graphics);
}
