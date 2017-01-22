use graphics::{self, Transformed};

use backend::gfx::ImageSize;

use ui::Resources;
use resources::Id;
use widget::DrawArgs;
use util::Dimensions;

pub struct ImageDrawable {
    pub image_id: Id,
    pub scale: Dimensions,
}
impl ImageDrawable {
    pub fn new(image_id: Id) -> Self {
        ImageDrawable {
            image_id: image_id,
            scale: Dimensions {
                width: 1.0,
                height: 1.0,
            },
        }
    }
    pub fn measure_image(&self, resources: &Resources) -> Dimensions {
        let img = resources.images.get(self.image_id).unwrap();
        img.get_size().into()
    }
    pub fn scale(&mut self, scale: Dimensions) {
        self.scale = scale;
    }
}

pub fn draw_image(draw_args: DrawArgs) {
    let DrawArgs { state, bounds, resources, context, graphics, .. } = draw_args;
    let state: &ImageDrawable = state.downcast_ref().unwrap();

    let img = resources.images.get(state.image_id).unwrap();
    let dims: Dimensions = img.get_size().into();
    let scale = bounds.dims() / dims;
    let image = graphics::image::Image::new();
    image.rect(bounds);
    let context = context.trans(bounds.left, bounds.top).scale(scale.width, scale.height);

    image.draw(img, &context.draw_state, context.transform, graphics);
}
