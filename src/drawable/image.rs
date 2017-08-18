use webrender_api::*;

use render::RenderBuilder;
use widget::drawable::Drawable;
use resources::{ImageId, resources};
use util::{self, Rect, RectExt, Size, SizeExt};

pub struct ImageDrawable {
    pub image: String,
    pub scale: Size,
}
impl ImageDrawable {
    pub fn new(image: &str) -> Self {
        ImageDrawable {
            image: image.to_owned(),
            scale: Size::new(1.0, 1.0),
        }
    }
    pub fn measure(&self) -> Size {
        Size::zero()
        /* let res = resources();
        let img = res.images.get(self.image_id).unwrap();
        Size::from_tuple(img.get_size()) */
    }
    pub fn scale(&mut self, scale: Size) {
        self.scale = scale;
    }
}
impl Drawable for ImageDrawable {
    fn draw(&mut self, bounds: Rect, _: Rect, renderer: &mut RenderBuilder) {
        let key = renderer.get_image(&self.image).key;
        renderer.builder.push_image(
            util::to_layout_rect(bounds),
            None,
            LayoutSize::zero(),
            LayoutSize::zero(),
            ImageRendering::Auto,
            key,
        );
        /*let res = resources();
        let img = res.images.get(self.image_id).unwrap();
        let dims = Size::from_tuple(img.get_size());
        let scale = Size::new(
            bounds.size.width / dims.width,
            bounds.size.height / dims.height,
        );
        let image = graphics::image::Image::new();
        image.rect(bounds.to_slice());
        let context = context.trans(bounds.left(), bounds.top()).scale(scale.width, scale.height);

        image.draw(img, &context.draw_state, context.transform, graphics);*/
    }
}
