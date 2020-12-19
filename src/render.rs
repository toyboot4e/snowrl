//! Rendering specialized for the [`World`]

use {
    rlbox::render::tiled as tiled_render,
    rokol::{
        app as ra,
        gfx::{self as rg, BakedResource},
    },
    snow2d::{
        gfx::{
            batcher::{draw::*, vertex::VertexData},
            geom2d::*,
            tex::{RenderTexture, Texture2dDrop},
            Color,
        },
        PassConfig, Snow2d,
    },
};

use crate::world::World;

const ALPHA_BLEND: rg::BlendState = rg::BlendState {
    enabled: true,
    src_factor_rgb: rg::BlendFactor::SrcAlpha as u32,
    dst_factor_rgb: rg::BlendFactor::OneMinusSrcAlpha as u32,
    op_rgb: 0,
    src_factor_alpha: rg::BlendFactor::One as u32,
    dst_factor_alpha: rg::BlendFactor::Zero as u32,
    op_alpha: 0,
    color_write_mask: 0,
    color_attachment_count: 1,
    color_format: 0,
    depth_format: 0,
    blend_color: [0.0; 4],
};

pub fn render_tiled(draw: &mut impl DrawApi, world: &World) {
    let bounds = Rect2f::from(([0.0, 0.0], ra::size_scaled()));
    tiled_render::render_tiled(draw, &world.map.tiled, &world.map.idmap, bounds.clone());
}

#[derive(Debug)]
pub struct FovRenderer {
    pa_trans: rg::PassAction,
    shadows: [RenderTexture; 2],
    /// Pipeline object for off-screen rendering with gausssian blur
    pip_gauss_ofs: rg::Pipeline,
}

impl FovRenderer {
    /// 1/4 off-screern rendering target
    fn create_shadow() -> RenderTexture {
        let inv_scale = 4.0;
        let mut screen_size = ra::size_scaled();
        screen_size[0] /= inv_scale;
        screen_size[1] /= inv_scale;
        RenderTexture::new(screen_size[0] as u32, screen_size[1] as u32)
    }

    pub fn new() -> Self {
        Self {
            pa_trans: rg::PassAction::clear(Color::BLACK.to_normalized_array()),
            shadows: [Self::create_shadow(), Self::create_shadow()],
            pip_gauss_ofs: rg::Pipeline::create(&rg::PipelineDesc {
                shader: snow2d::gfx::shaders::gauss(),
                index_type: rg::IndexType::UInt16 as u32,
                layout: VertexData::layout_desc(),
                blend: rg::BlendState {
                    depth_format: rg::PixelFormat::Depth as u32,
                    // ..ALPHA_BLEND
                    ..Default::default()
                },
                rasterizer: rg::RasterizerState {
                    // NOTE: our 2 renderer may output backward triangle
                    cull_mode: rg::CullMode::None as u32,
                    ..Default::default()
                },
                ..Default::default()
            }),
        }
    }

    // TODO: separate gaussian blur shader
    /// Render shadow texture
    pub fn render_ofs(&mut self, rdr: &mut Snow2d, world: &World) {
        // get shadow
        let mut offscreen = rdr.offscreen(
            &self.shadows[0],
            PassConfig {
                pa: &self.pa_trans,
                tfm: None,
                pip: None,
            },
        );

        let bounds = Rect2f::from(([0.0, 0.0], ra::size_scaled()));
        tiled_render::render_fov_shadows(
            &mut offscreen,
            &world.map.tiled,
            &world.player.fov,
            &bounds,
        );

        drop(offscreen);

        // apply gaussian blur
        self.pingpong(rdr);
    }

    fn pingpong(&mut self, rdr: &mut Snow2d) {
        // just to clear
        rdr.offscreen(
            &mut self.shadows[1],
            PassConfig {
                pa: &self.pa_trans,
                tfm: None,
                pip: Some(self.pip_gauss_ofs),
            },
        );

        for ix in 0..2 {
            // source shadow index
            let i = ix % 2;
            // target shadow index
            let j = (ix + 1) % 2;

            let mut draw = rdr.offscreen(
                &mut self.shadows[j],
                PassConfig {
                    pa: &rg::PassAction::NONE,
                    tfm: None,
                    pip: Some(self.pip_gauss_ofs),
                },
            );

            // horizontally or vertically
            unsafe {
                let ub_index = 1;
                let is_h: f32 = if ix % 2 == 0 { 1.0 } else { 0.0 };
                rg::apply_uniforms_as_bytes(rg::ShaderStage::Vs, ub_index, &is_h);
            }

            // write from one to the other
            draw.sprite(self.shadows[i].tex())
                // NOTE: we're using a orthogarphic projection matrix for the screen, so
                // use the screen size as the destination size
                .dst_size_px(ra::size_scaled());
        }
    }

    /// Writes shadow to the screen frame buffer
    pub fn blend_to_screen(&self, rdr: &mut Snow2d) {
        let mut screen = rdr.screen(PassConfig {
            pa: &rg::PassAction::NONE,
            tfm: None,
            pip: None,
        });

        screen
            .sprite(self.shadows[0].tex())
            .dst_size_px(ra::size_scaled());

        drop(screen);
    }
}
