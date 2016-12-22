
use backend::gfx::G2d;
use graphics;
use graphics::Context;
use graphics::types::Color;

use input::Event;
use super::util::*;

use cassowary::{Solver, Variable, Constraint};
use cassowary::WeightedRelation::*;
use cassowary::strength::*;

use super::text;
use backend::glyph::GlyphCache;
use backend::gfx::ImageSize;
use backend::glyph;
use text::Wrap;
use text::font;
use super::ui::Resources;

pub trait EventListener {
    fn handle_event(&self, event: &Event);
    fn matches(&self, event: &Event) -> bool {
        false
    }
}

pub trait WidgetDrawable {
    fn draw(&self,
            bounds: Rectangle,
            resources: &mut Resources,
            context: Context,
            graphics: &mut G2d);
    fn is_mouse_over(&self, mouse: Point, bounds: Rectangle) -> bool {
        point_inside_rect(mouse, bounds)
    }
}

pub struct RectDrawable {
    pub background: Color,
}
impl WidgetDrawable for RectDrawable {
    fn draw(&self,
            bounds: Rectangle,
            resources: &mut Resources,
            context: Context,
            graphics: &mut G2d) {
        graphics::Rectangle::new(self.background)
            .draw(bounds, &context.draw_state, context.transform, graphics);
    }
}

pub struct EllipseDrawable {
    pub background: Color,
}
impl WidgetDrawable for EllipseDrawable {
    fn draw(&self,
            bounds: Rectangle,
            resources: &mut Resources,
            context: Context,
            graphics: &mut G2d) {
        graphics::Ellipse::new(self.background)
            .draw(bounds, &context.draw_state, context.transform, graphics);
    }
    fn is_mouse_over(&self, mouse: Point, bounds: Rectangle) -> bool {
        let radius = Dimensions {
            width: bounds.width / 2.0,
            height: bounds.height / 2.0,
        };
        let center = Point {
            x: bounds.left + radius.width,
            y: bounds.top + radius.height,
        };
        point_inside_ellipse(mouse, center, radius)
    }
}

pub struct TextDrawable {
    pub text: String,
    pub font_id: font::Id,
    pub font_size: Scalar,
    pub text_color: Color,
    pub background_color: Color,
}
impl WidgetDrawable for TextDrawable {
    fn draw(&self,
            bounds: Rectangle,
            resources: &mut Resources,
            context: Context,
            graphics: &mut G2d) {

        graphics::Rectangle::new(self.background_color)
            .draw(bounds, &context.draw_state, context.transform, graphics);

        let GlyphCache { texture: ref mut text_texture_cache,
                         cache: ref mut glyph_cache,
                         ref mut vertex_data } = resources.glyph_cache;

        let font = resources.fonts.get(self.font_id).unwrap();
        let line_wrap = Wrap::Character;

        let positioned_glyphs = &text::get_positioned_glyphs(&self.text,
                                                             bounds,
                                                             font,
                                                             self.font_size,
                                                             line_wrap,
                                                             Align::Start,
                                                             Align::Start);

        // Queue the glyphs to be cached.
        for glyph in positioned_glyphs.iter() {
            glyph_cache.queue_glyph(self.font_id.index(), glyph.clone());
        }

        // Cache the glyphs within the GPU cache.
        glyph_cache.cache_queued(|rect, data| {
                glyph::cache_queued_glyphs(graphics, text_texture_cache, rect, data, vertex_data)
            })
            .unwrap();

        let tex_dim = {
            let (tex_w, tex_h) = text_texture_cache.get_size();
            Dimensions {
                width: tex_w as f64,
                height: tex_h as f64,
            }
        };

        let rectangles = positioned_glyphs.into_iter()
            .filter_map(|g| glyph_cache.rect_for(self.font_id.index(), g).ok().unwrap_or(None))
            .map(|(uv_rect, screen_rect)| {
                (map_rect_i32(screen_rect), map_rect_f32(uv_rect) * tex_dim)
            });
        // A re-usable buffer of rectangles describing the glyph's screen and texture positions.
        let mut glyph_rectangles = Vec::new();
        glyph_rectangles.extend(rectangles);
        graphics::image::draw_many(&glyph_rectangles,
                                   self.text_color,
                                   text_texture_cache,
                                   &context.draw_state,
                                   context.transform,
                                   graphics);
    }
}

pub struct WidgetLayout {
    pub left: Variable,
    pub right: Variable,
    pub top: Variable,
    pub bottom: Variable,
    pub constraints: Vec<Constraint>,
}
impl WidgetLayout {
    fn new() -> Self {
        WidgetLayout {
            left: Variable::new(),
            right: Variable::new(),
            top: Variable::new(),
            bottom: Variable::new(),
            constraints: Vec::new(),
        }
    }
    pub fn bounds(&self, solver: &mut Solver) -> Rectangle {
        Rectangle {
            left: solver.get_value(self.left),
            top: solver.get_value(self.top),
            width: solver.get_value(self.right) - solver.get_value(self.left),
            height: solver.get_value(self.bottom) - solver.get_value(self.top),
        }
    }
    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }
    pub fn add_constraints(&mut self, constraints: &[Constraint]) {
        self.constraints.extend_from_slice(constraints);
    }
    pub fn width(&mut self, width: Scalar, strength: f64) {
        self.constraints.push(self.right - self.left | EQ(strength) | width)
    }
    pub fn height(&mut self, height: Scalar, strength: f64) {
        self.constraints.push(self.bottom - self.top | EQ(strength) | height)
    }
    pub fn bound_by(&mut self, layout: &WidgetLayout) {
        let constraints = [layout.left | GE(REQUIRED) | self.left,
                           layout.top | GE(REQUIRED) | self.top,
                           layout.right | LE(REQUIRED) | self.right,
                           layout.bottom | LE(REQUIRED) | self.bottom];
        self.add_constraints(&constraints);
    }
}

pub struct Widget {
    pub drawable: Option<Box<WidgetDrawable>>,
    pub layout: WidgetLayout,
    pub listeners: Vec<Box<EventListener>>,
}

impl Widget {
    pub fn new(drawable: Option<Box<WidgetDrawable>>) -> Self {
        Widget {
            drawable: drawable,
            layout: WidgetLayout::new(),
            listeners: Vec::new(),
        }
    }
    pub fn print(&self, solver: &mut Solver) {
        println!("{:?}", self.layout.bounds(solver));
    }
    pub fn draw(&self,
                resources: &mut Resources,
                solver: &mut Solver,
                c: Context,
                g: &mut G2d) {
        if let Some(ref drawable) = self.drawable {
            drawable.draw(self.layout.bounds(solver), resources, c, g);
        }
    }
    pub fn is_mouse_over(&self, solver: &mut Solver, mouse: Point) -> bool {
        let bounds = self.layout.bounds(solver);
        if let Some(ref drawable) = self.drawable {
            drawable.is_mouse_over(mouse, bounds)
        } else {
            point_inside_rect(mouse, bounds)
        }
    }
}
