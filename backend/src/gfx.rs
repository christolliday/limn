//! Contains the GfxContext wrapper for convenient integration with `backend::piston::Window`

extern crate gfx;

use shader_version::OpenGL;
use gfx_graphics::{Gfx2d, GfxGraphics};
use self::gfx::Device;
use self::gfx::format::{DepthStencil, Format, Formatted, Srgba8};
use self::gfx::memory::Typed;

use glutin;

use texture;
use gfx_device_gl;
use super::glyph::GlyphCache;
use graphics::Viewport;

pub use graphics::{Context, DrawState, Graphics, ImageSize, Transformed};
pub use gfx_graphics::{GlyphError, Texture, TextureSettings, Flip};

/// Actual gfx::Stream implementation carried by the window.
pub type GfxEncoder = gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>;
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

fn create_main_targets(dim: gfx::texture::Dimensions) ->
    (gfx::handle::RenderTargetView<gfx_device_gl::Resources, gfx::format::Srgba8>,
     gfx::handle::DepthStencilView<gfx_device_gl::Resources, gfx::format::DepthStencil>)
 {
    let color_format: Format = <Srgba8 as Formatted>::get_format();
    let depth_format: Format = <DepthStencil as Formatted>::get_format();
    let (output_color, output_stencil) =
        gfx_device_gl::create_main_targets_raw(dim, color_format.0, depth_format.0);
    let output_color = Typed::new(output_color);
    let output_stencil = Typed::new(output_stencil);
    (output_color, output_stencil)
}

impl GfxContext {
    /// Constructor for a new `GfxContext`
    pub fn new(window: &mut glutin::Window, opengl: OpenGL, samples: u8) -> Self {
        let (device, mut factory) = gfx_device_gl::create(|s| window.get_proc_address(s) as *const _);

        let draw_size: (u32, u32) = window.get_inner_size_pixels().unwrap_or((0, 0));
        let (output_color, output_stencil) = {
            let aa = samples as gfx::texture::NumSamples;
            let dim = (draw_size.0 as u16, draw_size.1 as u16, 1, aa.into());
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
    pub fn draw_2d<F, U>(&mut self, f: F, viewport: Viewport) -> U where
        F: FnOnce(Context, &mut G2d) -> U
    {
        let res = self.g2d.draw(
            &mut self.encoder,
            &self.output_color,
            &self.output_stencil,
            viewport,
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
    pub fn check_resize(&mut self, draw_size: (u32, u32)) {
        let dim = self.output_color.raw().get_dimensions();
        let (w, h) = (dim.0, dim.1);
        if w != draw_size.0 as u16 || h != draw_size.1 as u16 {
            let dim = (draw_size.0 as u16,
                       draw_size.1 as u16,
                       dim.2, dim.3);
            let (output_color, output_stencil) = create_main_targets(dim);
            self.output_color = output_color;
            self.output_stencil = output_stencil;
        }
    }
}