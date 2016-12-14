//! Contains the GfxContext wrapper for convenient integration with `backend::piston::Window`

extern crate window as pistoncore_window;
extern crate graphics as piston_graphics;
extern crate gfx;
extern crate gfx_core;
extern crate gfx_device_gl;
extern crate gfx_graphics;
extern crate shader_version;
extern crate texture;

use self::shader_version::OpenGL;
use self::gfx_graphics::{Gfx2d, GfxGraphics};
use self::gfx_core::factory::Typed;
use self::gfx::Device;

use self::pistoncore_window::{OpenGLWindow, Size};
use piston_input::RenderArgs;

pub use self::piston_graphics::{Context, DrawState, Graphics, ImageSize, Transformed};
pub use self::gfx_graphics::{GlyphError, Texture, TextureSettings, Flip};

/// Actual gfx::Stream implementation carried by the window.
pub type GfxEncoder = gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>;
/// Glyph cache.
pub type Glyphs = gfx_graphics::GlyphCache<gfx_device_gl::Resources, gfx_device_gl::Factory>;
/// 2D graphics.
pub type G2d<'a> = GfxGraphics<'a, gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>;
/// Texture type compatible with `G2d`.
pub type G2dTexture<'a> = Texture<gfx_device_gl::Resources>;

/// Contains state used by Gfx to draw. Can be stored within a window.
pub struct GfxContext {
    /// GFX encoder.
    pub encoder: GfxEncoder,
    /// GFX device.
    pub device: gfx_device_gl::Device,
    /// Output frame buffer.
    pub output_color: gfx::handle::RenderTargetView<
        gfx_device_gl::Resources, gfx::format::Srgba8>,
    /// Output stencil buffer.
    pub output_stencil: gfx::handle::DepthStencilView<
        gfx_device_gl::Resources, gfx::format::DepthStencil>,
    /// Gfx2d.
    pub g2d: Gfx2d<gfx_device_gl::Resources>,
    /// The factory that was created along with the device.
    pub factory: gfx_device_gl::Factory,
}

fn create_main_targets(dim: gfx::tex::Dimensions) ->
    (gfx::handle::RenderTargetView<gfx_device_gl::Resources, gfx::format::Srgba8>,
     gfx::handle::DepthStencilView<gfx_device_gl::Resources, gfx::format::DepthStencil>)
 {
    use self::gfx_core::factory::Typed;
    use self::gfx::format::{DepthStencil, Format, Formatted, Srgba8};

    let color_format: Format = <Srgba8 as Formatted>::get_format();
    let depth_format: Format = <DepthStencil as Formatted>::get_format();
    let (output_color, output_stencil) =
        gfx_device_gl::create_main_targets_raw(dim,
                                               color_format.0,
                                               depth_format.0);
    let output_color = Typed::new(output_color);
    let output_stencil = Typed::new(output_stencil);
    (output_color, output_stencil)
}

impl GfxContext {
    /// Constructor for a new `GfxContext`
    pub fn new<W>(window: &mut W, opengl: OpenGL, samples: u8) -> Self
        where W: OpenGLWindow 
    {
        let (device, mut factory) = gfx_device_gl::create(|s| window.get_proc_address(s) as *const _);

        let draw_size = window.draw_size();
        let (output_color, output_stencil) = {
            let aa = samples as gfx::tex::NumSamples;
            let dim = (draw_size.width as u16, draw_size.height as u16,
                       1, aa.into());
            create_main_targets(dim)
        };

        let g2d = Gfx2d::new(opengl, &mut factory);
        let encoder = factory.create_command_buffer().into();
        GfxContext {
            encoder: encoder,
            device: device,
            output_color: output_color,
            output_stencil: output_stencil,
            g2d: g2d,
            factory: factory,
        }
    }

    /// Renders 2D graphics.
    pub fn draw_2d<F, U>(&mut self, f: F, args: RenderArgs) -> U where
        F: FnOnce(Context, &mut G2d) -> U
    {
        let res = self.g2d.draw(
            &mut self.encoder,
            &self.output_color,
            &self.output_stencil,
            args.viewport(),
            f
        );
        self.encoder.flush(&mut self.device);
        res
    }

    /// Called after frame is rendered to cleanup after gfx device.
    pub fn after_render(&mut self) {
        self.device.cleanup();
    }

    /// Check whether window has resized and update the output.
    pub fn check_resize(&mut self, draw_size: Size) {
        let dim = self.output_color.raw().get_dimensions();
        let (w, h) = (dim.0, dim.1);
        if w != draw_size.width as u16 || h != draw_size.height as u16 {
            let dim = (draw_size.width as u16,
                       draw_size.height as u16,
                       dim.2, dim.3);
            let (output_color, output_stencil) = create_main_targets(dim);
            self.output_color = output_color;
            self.output_stencil = output_stencil;
        }
    }
}