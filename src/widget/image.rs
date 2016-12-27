
use graphics;
use super::super::util::*;
use super::super::ui::Resources;
use backend::gfx::G2d;
use graphics::Context;
use graphics::types::Color;
use graphics::Transformed;
use resources::Id;
use std::any::Any;

pub struct ImageDrawable {
    pub image_id: Id,
}

pub fn draw_image(state: &Any,
            bounds: Rectangle,
            resources: &mut Resources,
            context: Context,
            graphics: &mut G2d) {
    let state: &ImageDrawable = state.downcast_ref().unwrap();

        let img = resources.images.get(state.image_id).unwrap();
        let mut image = graphics::image::Image::new();
        image.rect(bounds);
        image.draw(img, &context.draw_state, context.transform, graphics);
}