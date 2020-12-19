//! Rendering specialized for the [`World`]

use {
    rlbox::{
        render::tiled as tiled_render,
        rl::{self, fov::FovData, fow::FowData, grid2d::*, rlmap::TiledRlMap},
    },
    rokol::{
        app as ra,
        gfx::{self as rg, BakedResource},
    },
    snow2d::{
        gfx::{
            batcher::{draw::*, vertex::VertexData},
            geom2d::*,
            tex::Texture2dDrop,
            Color,
        },
        PassConfig, RenderTexture, Snow2d,
    },
    std::path::{Path, PathBuf},
};

use crate::world::{World, WorldContext};

const ALPHA_BLEND: rg::BlendState = rg::BlendState {
    enabled: true,
    src_factor_rgb: rg::BlendFactor::SrcAlpha as u32,
    dst_factor_rgb: rg::BlendFactor::OneMinusSrcAlpha as u32,
    op_rgb: 0,
    src_factor_alpha: 0,
    dst_factor_alpha: 0,
    op_alpha: 0,
    color_write_mask: 0,
    color_attachment_count: 0,
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
    pub shadow: RenderTexture,
    pip: rg::Pipeline,
}

impl FovRenderer {
    pub fn new() -> Self {
        let shadow = {
            // 1/4 off-screern rendering target
            let inv_scale = 4.0;
            let mut screen_size = ra::size_scaled();
            screen_size[0] /= inv_scale;
            screen_size[1] /= inv_scale;
            RenderTexture::new(screen_size[0] as u32, screen_size[1] as u32)
        };

        let pip = rg::Pipeline::create(&rg::PipelineDesc {
            shader: snow2d::gfx::shaders::aver(),
            index_type: rg::IndexType::UInt16 as u32,
            layout: VertexData::layout_desc(),
            blend: ALPHA_BLEND,
            rasterizer: rg::RasterizerState {
                // NOTE: our 2 renderer may output backward triangle
                cull_mode: rg::CullMode::None as u32,
                ..Default::default()
            },
            ..Default::default()
        });

        Self {
            pa_trans: rg::PassAction::clear(Color::BLACK.to_normalized_array()),
            shadow,
            pip,
        }
    }

    fn tex(&self) -> &Texture2dDrop {
        &self.shadow.tex()
    }

    /// Updates internal shadow texture
    pub fn render(&mut self, rdr: &mut Snow2d, world: &World) {
        let mut offscreen = rdr.offscreen(
            &self.shadow,
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
    }

    /// Writes shadow to the screen frame buffer
    pub fn blend_to_screen(&self, rdr: &mut Snow2d) {
        let mut screen = rdr.screen(PassConfig {
            pa: &rg::PassAction::NONE,
            tfm: None,
            pip: Some(self.pip),
        });

        screen.sprite(self.tex()).dst_size_px(ra::size_scaled());

        drop(screen);
    }
}
