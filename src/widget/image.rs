
use graphics;
use super::super::util::*;
use super::super::ui::Resources;
use backend::gfx::G2d;
use graphics::Context;
use graphics::types::Color;
use graphics::Transformed;
use super::WidgetDrawable;
use resources::Id;

pub struct ImageDrawable {
    pub image_id: Id,
}

impl WidgetDrawable for ImageDrawable {
    fn draw(&self,
            bounds: Rectangle,
            resources: &mut Resources,
            context: Context,
            graphics: &mut G2d) {
        let img = resources.images.get(self.image_id).unwrap();
        let mut image = graphics::image::Image::new();
        image.rect(bounds);
        image.draw(img, &context.draw_state, context.transform, graphics);
    }
}