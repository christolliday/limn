use graphics::{self, Context, Transformed};

use backend::gfx::{ImageSize, G2d};
use backend::glyph::GlyphCache;

use resources::{ImageId, resources};
use widget::drawable::Drawable;
use util::{Dimensions, Rectangle};

pub struct ImageDrawable {
    pub image_id: ImageId,
    pub scale: Dimensions,
}
impl ImageDrawable {
    pub fn new(image_id: ImageId) -> Self {
        ImageDrawable {
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
impl Drawable for ImageDrawable {
    fn draw(&mut self, bounds: Rectangle, _: Rectangle, _: &mut GlyphCache, context: Context, graphics: &mut G2d) {
        let res = resources();
        let img = res.images.get(self.image_id).unwrap();
        let dims: Dimensions = img.get_size().into();
        let scale = bounds.dims() / dims;
        let image = graphics::image::Image::new();
        image.rect(bounds);
        let context = context.trans(bounds.left, bounds.top).scale(scale.width, scale.height);

        image.draw(img, &context.draw_state, context.transform, graphics);
    }
}
