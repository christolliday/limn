use graphics::{self, Transformed};

use backend::gfx::ImageSize;

use resources::{ImageId, resources};
use widget::drawable::{Drawable, DrawArgs};
use util::Dimensions;

pub fn image_drawable(image_id: ImageId) -> Drawable {
    let draw_state = ImageDrawState::new(image_id);
    Drawable::new(draw_state, draw_image)
}
pub struct ImageDrawState {
    pub image_id: ImageId,
    pub scale: Dimensions,
}
impl ImageDrawState {
    pub fn new(image_id: ImageId) -> Self {
        ImageDrawState {
            image_id: image_id,
            scale: Dimensions {
                width: 1.0,
                height: 1.0,
            },
        }
    }
    pub fn measure(&self) -> Dimensions {
        let res = resources();
        let img = res.images.get(self.image_id).unwrap();
        img.get_size().into()
    }
    pub fn scale(&mut self, scale: Dimensions) {
        self.scale = scale;
    }
}

pub fn measure(drawable: &Drawable) -> Dimensions {
    let draw_state: &ImageDrawState = drawable.state();
    draw_state.measure()
}

pub fn draw_image(args: DrawArgs<ImageDrawState>) {
    let DrawArgs { state, bounds, context, graphics, .. } = args;

    let res = resources();
    let img = res.images.get(state.image_id).unwrap();
    let dims: Dimensions = img.get_size().into();
    let scale = bounds.dims() / dims;
    let image = graphics::image::Image::new();
    image.rect(bounds);
    let context = context.trans(bounds.left, bounds.top).scale(scale.width, scale.height);

    image.draw(img, &context.draw_state, context.transform, graphics);
}
