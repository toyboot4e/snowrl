//! Rendering specialized for the [`World`]

use {
    rlbox::{render::tiled as tiled_render, rl::fov::FovData},
    rokol::{
        app as ra,
        gfx::{self as rg, BakedResource},
    },
    snow2d::{
        gfx::{
            batcher::{draw::*, vertex::VertexData},
            geom2d::*,
            tex::RenderTexture,
            Color,
        },
        PassConfig, Snow2d,
    },
};

use crate::world::World;

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
    // FoV rendering states
    fov_prev: FovData,
    fov_blend_factor: f32,
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
                    ..Default::default()
                },
                rasterizer: rg::RasterizerState {
                    // NOTE: our 2 renderer may output backward triangle
                    cull_mode: rg::CullMode::None as u32,
                    ..Default::default()
                },
                ..Default::default()
            }),
            fov_prev: Default::default(),
            fov_blend_factor: 0.0,
        }
    }

    pub fn force_set_fov(&mut self, fov: &FovData) {
        self.fov_prev = fov.clone();
        self.fov_blend_factor = 0.0;
    }

    pub fn before_update_fov(&mut self, fov: &FovData) {
        self.fov_prev = fov.clone();
        self.fov_blend_factor = 0.0;
    }

    pub fn update(&mut self, dt: std::time::Duration) {
        // advance FoV blend factor
        self.fov_blend_factor += dt.as_secs_f32() / crate::consts::WALK_TIME;
        if self.fov_blend_factor >= 1.0 {
            self.fov_blend_factor = 1.0;
        }
    }

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

        // TODO: use camera
        let bounds = Rect2f::from(([0.0, 0.0], ra::size_scaled()));

        tiled_render::render_fov_shadows_blend(
            &mut offscreen,
            &world.map.tiled,
            &bounds,
            &world.player().fov,
            &self.fov_prev,
            self.fov_blend_factor,
        );

        drop(offscreen);

        // apply gaussian blur
        self.pingpong(rdr);
    }

    /// Apply gaussian blur
    fn pingpong(&mut self, rdr: &mut Snow2d) {
        // 5 times
        for _ in 0..5 {
            // pingpong blur
            for ix in 0..2 {
                // source shadow index
                let i = ix % 2;
                // target shadow index
                let j = (ix + 1) % 2;

                self.blur(rdr, ix == 0, i, j);
            }
        }
    }

    #[inline]
    fn blur(&mut self, rdr: &mut Snow2d, is_h: bool, from: usize, to: usize) {
        let mut draw = rdr.offscreen(
            &mut self.shadows[to],
            PassConfig {
                pa: &rg::PassAction::NONE,
                tfm: None,
                pip: Some(self.pip_gauss_ofs),
            },
        );

        // horizontally or vertically
        unsafe {
            let ub_index = 1;
            let uniform: f32 = if is_h { 1.0 } else { 0.0 };
            rg::apply_uniforms_as_bytes(rg::ShaderStage::Vs, ub_index, &uniform);
        }

        // write from one to the other
        draw.sprite(self.shadows[from].tex())
            // NOTE: we're using a orthogarphic projection matrix for the screen, so
            // use the screen size as the destination size
            .dst_size_px(ra::size_scaled());
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
