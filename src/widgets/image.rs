use graphics::{self, Transformed};

use backend::gfx::ImageSize;

use resources::{Id, resources};
use widget::{Drawable, DrawArgs};
use util::Dimensions;

pub fn image_drawable(image_id: Id) -> Drawable {
    let draw_state = ImageDrawState::new(image_id);
    Drawable::new(Box::new(draw_state), draw_image)
}
pub struct ImageDrawState {
    pub image_id: Id,
    pub scale: Dimensions,
}
impl ImageDrawState {
    pub fn new(image_id: Id) -> Self {
        ImageDrawState {
            image_id: image_id,
            scale: Dimensions {
                width: 1.0,
                height: 1.0,
            },
        }
    }
    pub fn measure_image(&self) -> Dimensions {
        let res = resources();
        let img = res.images.get(self.image_id).unwrap();
        img.get_size().into()
    }
    pub fn scale(&mut self, scale: Dimensions) {
        self.scale = scale;
    }
}

pub fn measure_image(drawable: &Drawable) -> Dimensions {
    let draw_state: &ImageDrawState = drawable.state();
    draw_state.measure_image()
}

pub fn draw_image(draw_args: DrawArgs) {
    let DrawArgs { state, bounds, context, graphics, .. } = draw_args;
    let state: &ImageDrawState = state.downcast_ref().unwrap();

    let res = resources();
    let img = res.images.get(state.image_id).unwrap();
    let dims: Dimensions = img.get_size().into();
    let scale = bounds.dims() / dims;
    let image = graphics::image::Image::new();
    image.rect(bounds);
    let context = context.trans(bounds.left, bounds.top).scale(scale.width, scale.height);

    image.draw(img, &context.draw_state, context.transform, graphics);
}
