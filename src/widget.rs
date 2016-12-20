
use backend::gfx::G2d;
use graphics;
use graphics::Context;
use graphics::types::{Color};

use input::Event;
use super::util::*;

use cassowary::{ Solver, Variable, Constraint };
use cassowary::WeightedRelation::*;
use cassowary::strength::*;

use super::ui::Ui;
use super::text::Text;
use super::text;
use backend::glyph::GlyphCache;
use backend::gfx::G2dTexture;
use rusttype;
use backend::gfx::ImageSize;
use find_folder;
use backend::glyph;
use graphics::Transformed;
use text::PositionedGlyph;

pub trait EventListener {
    fn handle_event(&self, event: &Event);
    fn matches(&self, event: &Event) -> bool {
        false
    }
}

pub trait WidgetDrawable {
    fn draw(&self, fonts: &text::font::Map, glyph_cache: &mut GlyphCache, bounds: Rectangle, context: Context, graphics: &mut G2d);
    fn is_mouse_over(&self, mouse: Point, bounds: Rectangle) -> bool {
        point_inside_rect(mouse, bounds)
    }
}

pub struct RectDrawable {
    pub background: Color,
}
impl WidgetDrawable for RectDrawable {
    fn draw(&self, fonts: &text::font::Map, glyph_cache: &mut GlyphCache, bounds: Rectangle, context: Context, graphics: &mut G2d) {
        graphics::Rectangle::new(self.background).draw(bounds, &context.draw_state, context.transform, graphics);
    }
}

pub struct EllipseDrawable {
    pub background: Color,
}
impl WidgetDrawable for EllipseDrawable {
    fn draw(&self, fonts: &text::font::Map, glyph_cache: &mut GlyphCache, bounds: Rectangle, context: Context, graphics: &mut G2d) {
        graphics::Ellipse::new(self.background).draw(bounds, &context.draw_state, context.transform, graphics);
    }
    fn is_mouse_over(&self, mouse: Point, bounds: Rectangle) -> bool {
        let radius = Dimensions { width: bounds.width / 2.0, height: bounds.height / 2.0 };
        let center = Point { x: bounds.left + radius.width, y: bounds.top + radius.height };
        point_inside_ellipse(mouse, center, radius)
    }
}

pub struct TextDrawable {
    pub font_id: text::font::Id,
}
pub struct TextPrimitive<'a> {
    /// The colour of the `Text`.
    color: Color,
    /// All glyphs within the `Text` laid out in their correct positions in order from top-left
    /// to bottom right.
    text: Text<'a>,
    /// The unique identifier for the font, useful for the `glyph_cache.rect_for(id, glyph)`
    /// method when using the `conrod::text::GlyphCache` (rusttype's GPU `Cache`).
    font_id: text::font::Id,
}
impl WidgetDrawable for TextDrawable {
    fn draw(&self, fonts: &text::font::Map, glyph_cache: &mut GlyphCache, bounds: Rectangle, context: Context, graphics: &mut G2d) {

        let view_size = context.get_view_size();
        //let GlyphCache { ref mut text_texture_cache, ref mut glyph_cache, ref mut vertex_data } = *ui.glyph_cache;
        let GlyphCache { ref mut texture, ref mut cache, ref mut vertex_data } = *glyph_cache;

        let text_texture_cache = texture;
        let glyph_cache = cache;

        // A function used for caching glyphs from `Text` widgets.
        let mut cache_queued_glyphs_fn = |graphics: &mut G2d,
                                    cache: &mut G2dTexture<'static>,
                                    rect: rusttype::Rect<u32>,
                                    data: &[u8]|
        {
            glyph::cache_queued_glyphs(graphics, cache, rect, data, vertex_data);
        };
        let mut positioned_glyphs = Vec::new();

        let font_id = self.font_id;
        let font = fonts.get(font_id).unwrap();
        let color = [1.0,1.0,1.0,1.0];
        let font_size = 12;
        let line_spacing = 1.0;
        let window_dim = Dimensions { width: 400.0, height: 720.0 };

        let text_string = "Testing";
        let line_infos: Vec<text::line::Info> = text::line::infos(text_string, font, font_size).collect();
        let text_a = Text {
            positioned_glyphs: &mut positioned_glyphs,
            window_dim: window_dim.into(),
            text: &text_string,
            line_infos: &line_infos,
            font: font,
            font_size: font_size,
            rect: bounds,
            x_align: Align::Start,
            y_align: Align::End,
            line_spacing: line_spacing,
        };

        let text = TextPrimitive {
            color: color,
            text: text_a,
            font_id: font_id,
        };
        //let text = get_text(fonts, self.font_id, bounds, &mut positioned_glyphs, line_infos);
        let font_id = text.font_id;
        let color = text.color;
        //let ref text = self.text;
        // A re-usable buffer of rectangles describing the glyph's screen and texture positions.
        let mut glyph_rectangles = Vec::new();

        //Rectangle::new(self.color).draw(bounds, &c.draw_state, c.transform, g);

        // Retrieve the "dots per inch" factor by dividing the window width by the view.
        //
        // TODO: Perhaps this should be a method on the `Context` type?
        let dpi_factor = context.viewport
            .map(|v| v.window_size[0] as f32 / view_size[0] as f32)
            .unwrap_or(1.0);
        let positioned_glyphs = text.text.positioned_glyphs(dpi_factor);
        // Re-orient the context to top-left origin with *y* facing downwards, as the
        // `positioned_glyphs` yield pixel positioning.
        let context = context.scale(1.0, -1.0).trans(-view_size[0] / 2.0, -view_size[1] / 2.0);

        // Queue the glyphs to be cached.
        for glyph in positioned_glyphs.iter() {
            glyph_cache.queue_glyph(font_id.index(), glyph.clone());
        }

        // Cache the glyphs within the GPU cache.
        glyph_cache.cache_queued(|rect, data| {
            cache_queued_glyphs_fn(graphics, text_texture_cache, rect, data)
        }).unwrap();

        let cache_id = font_id.index();
        let (tex_w, tex_h) = text_texture_cache.get_size();
        //let color = color.to_fsa();

        let rectangles = positioned_glyphs.into_iter()
            .filter_map(|g| glyph_cache.rect_for(cache_id, g).ok().unwrap_or(None))
            .map(|(uv_rect, screen_rect)| {
                let rectangle = {
                    let div_dpi_factor = |s| (s as f32 / dpi_factor as f32) as f64;
                    let left = div_dpi_factor(screen_rect.min.x);
                    let top = div_dpi_factor(screen_rect.min.y);
                    let right = div_dpi_factor(screen_rect.max.x);
                    let bottom = div_dpi_factor(screen_rect.max.y);
                    let w = right - left;
                    let h = bottom - top;
                    [left, top, w, h]
                };
                let source_rectangle = {
                    let x = (uv_rect.min.x * tex_w as f32) as f64;
                    let y = (uv_rect.min.y * tex_h as f32) as f64;
                    let w = ((uv_rect.max.x - uv_rect.min.x) * tex_w as f32) as f64;
                    let h = ((uv_rect.max.y - uv_rect.min.y) * tex_h as f32) as f64;
                    [x, y, w, h]
                };
                (rectangle, source_rectangle)
            });
        glyph_rectangles.clear();
        glyph_rectangles.extend(rectangles);
        graphics::image::draw_many(&glyph_rectangles,
                                          color,
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
    // layout
    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }
    pub fn add_constraints(&mut self, constraints: &[Constraint]) {
        self.constraints.extend_from_slice(constraints);
    }
    pub fn width(&mut self, width: Scalar, strength: f64) {
        self.constraints.push(self.right - self.left |EQ(strength)| width)
    }
    pub fn height(&mut self, height: Scalar, strength: f64) {
        self.constraints.push(self.bottom - self.top |EQ(strength)| height)
    }
    pub fn bound_by(&mut self, layout: &WidgetLayout) {
        let constraints = [
            layout.left |GE(REQUIRED)| self.left,
            layout.top |GE(REQUIRED)| self.top,
            layout.right |LE(REQUIRED)| self.right,
            layout.bottom |LE(REQUIRED)| self.bottom,
        ];
        self.add_constraints(&constraints);
    }
}

pub struct Widget {
    pub drawable: Option<Box<WidgetDrawable>>,
    pub layout: WidgetLayout,
    pub listeners: Vec<Box<EventListener>>
}

impl Widget  {
    pub fn new(drawable: Option<Box<WidgetDrawable>>) -> Self {
        Widget {
            drawable: drawable,
            layout: WidgetLayout::new(),
            listeners: Vec::new(),
        }
    }
    pub fn print(&self, solver: &mut Solver) {
        println!("{:?}",
            self.layout.bounds(solver));
    }
    pub fn draw(&self, fonts: &text::font::Map, glyph_cache: &mut GlyphCache, solver: &mut Solver, c: Context, g: &mut G2d) {
        if let Some(ref drawable) = self.drawable {
            drawable.draw(fonts, glyph_cache, self.layout.bounds(solver), c, g);
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