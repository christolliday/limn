use graphics::{self, Context, Transformed};

use backend::gfx::{ImageSize, G2d};
use backend::glyph::GlyphCache;

use widget::drawable::Drawable;
use resources::{ImageId, resources};
use util::{Rect, RectExt, Size, SizeExt};

pub struct ImageDrawable {
    pub image_id: ImageId,
    pub scale: Size,
}
impl ImageDrawable {
    pub fn new(image_id: ImageId) -> Self {
        ImageDrawable {
            image_id: image_id,
            scale: Size::new(1.0, 1.0),
        }
    }
    pub fn measure(&self) -> Size {
        let res = resources();
        let img = res.images.get(self.image_id).unwrap();
        Size::from_tuple(img.get_size())
    }
    pub fn scale(&mut self, scale: Size) {
        self.scale = scale;
    }
}
impl Drawable for ImageDrawable {
    fn draw(&mut self, bounds: Rect, _: Rect, _: &mut GlyphCache, context: Context, graphics: &mut G2d) {
        let res = resources();
        let img = res.images.get(self.image_id).unwrap();
        let dims = Size::from_tuple(img.get_size());
        let scale = Size::new(
            bounds.size.width / dims.width,
            bounds.size.height / dims.height,
        );
        let image = graphics::image::Image::new();
        image.rect(bounds.to_slice());
        let context = context.trans(bounds.left(), bounds.top()).scale(scale.width, scale.height);

        image.draw(img, &context.draw_state, context.transform, graphics);
    }
}
