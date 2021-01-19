//! The game world renderers

use {
    rlbox::render::tiled as tiled_render,
    rokol::{
        app as ra,
        gfx::{self as rg, BakedResource},
    },
    snow2d::gfx::{
        batch::{mesh::DynamicMesh, vertex::VertexData},
        draw::*,
        geom2d::*,
        tex::RenderTexture,
        Color, PassConfig, Snow2d,
    },
    std::time::Instant,
};

use crate::world::World;

#[derive(Debug)]
pub struct FovRenderer {
    pa_trans: rg::PassAction,
    /// Shadow textures for gaussian blur
    shadows: [RenderTexture; 2],
    /// Pipeline object for off-screen rendering with gausssian blur
    pip_gauss_ofs: rg::Pipeline,
}

impl Default for FovRenderer {
    fn default() -> Self {
        Self {
            pa_trans: rg::PassAction::clear(Color::BLACK.to_normalized_array()),
            shadows: [Self::create_shadow(), Self::create_shadow()],
            pip_gauss_ofs: rg::Pipeline::create(&rg::PipelineDesc {
                shader: snow2d::gfx::shaders::gauss(),
                index_type: rg::IndexType::UInt16 as u32,
                layout: VertexData::layout_desc(),
                blend: rg::BlendState {
                    // for off-screen rendering:
                    depth_format: rg::PixelFormat::Depth as u32,
                    ..Default::default()
                },
                rasterizer: rg::RasterizerState {
                    // NOTE: our renderer may output backward triangle
                    cull_mode: rg::CullMode::None as u32,
                    ..Default::default()
                },
                ..Default::default()
            }),
        }
    }
}

impl FovRenderer {
    /// Creates 1/4 off-screern rendering target
    fn create_shadow() -> RenderTexture {
        let inv_scale = 4.0;
        let mut screen_size = ra::size_scaled();
        screen_size[0] /= inv_scale;
        screen_size[1] /= inv_scale;
        RenderTexture::new(screen_size[0] as u32, screen_size[1] as u32)
    }

    /// Renders shadow texture (don't forget to use it later)
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

        tiled_render::render_fov_fow_blend(
            &mut offscreen,
            &world.map.tiled,
            &bounds,
            &world.shadow.fov.a,
            &world.shadow.fov.b,
            world.shadow.dt.get(),
            &world.shadow.fow.a,
            &world.shadow.fow.b,
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

        self.blend_to_target(&mut screen);

        drop(screen);
    }

    /// Writes shadow to the screen frame buffer
    pub fn blend_to_target(&self, target: &mut impl DrawApi) {
        target
            .sprite(self.shadows[0].tex())
            .dst_size_px(ra::size_scaled());
    }
}

/// Position-ony vertex data
#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct PosVert {
    pub pos: [f32; 2],
}

impl PosVert {
    pub fn layout_desc() -> rg::LayoutDesc {
        let mut desc = rg::LayoutDesc::default();
        desc.attrs[0].format = rg::VertexFormat::Float2 as u32;
        desc
    }
}

const M_INV_Y: glam::Mat4 = glam::const_mat4!(
    [1.0, 0.0, 0.0, 0.0],
    [0.0, -1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0]
);

#[derive(Debug)]
pub struct SnowRenderer {
    pa_trans: rg::PassAction,
    pip_snow: rg::Pipeline,
    start_time: Instant,
    mesh: DynamicMesh<PosVert>,
}

impl Default for SnowRenderer {
    fn default() -> Self {
        Self {
            pa_trans: rg::PassAction::clear(Color::BLACK.to_normalized_array()),
            pip_snow: rg::Pipeline::create(&rg::PipelineDesc {
                shader: snow2d::gfx::shaders::snow(),
                index_type: rg::IndexType::UInt16 as u32,
                layout: PosVert::layout_desc(),
                blend: rg::BlendState {
                    // alpha blending for on-screen rendering
                    enabled: true,
                    src_factor_rgb: rg::BlendFactor::SrcAlpha as u32,
                    dst_factor_rgb: rg::BlendFactor::OneMinusSrcAlpha as u32,
                    src_factor_alpha: rg::BlendFactor::One as u32,
                    dst_factor_alpha: rg::BlendFactor::Zero as u32,
                    ..Default::default()
                },
                rasterizer: rg::RasterizerState {
                    // NOTE: our renderer may output backward triangle
                    cull_mode: rg::CullMode::None as u32,
                    ..Default::default()
                },
                ..Default::default()
            }),
            start_time: Instant::now(),
            mesh: DynamicMesh::new_16(vec![PosVert::default(); 4], &[0, 1, 2, 3, 1, 2]),
        }
    }
}

impl SnowRenderer {
    pub fn render(&mut self) {
        rg::begin_default_pass(&rg::PassAction::NONE, ra::width(), ra::height());
        rg::apply_pipeline(self.pip_snow);

        // set uniforms
        unsafe {
            // VS
            rg::apply_uniforms_as_bytes(rg::ShaderStage::Vs, 0, &{
                let proj = glam::Mat4::orthographic_rh_gl(0.0, 1280.0, 720.0, 0.0, 0.0, 1.0);
                M_INV_Y * proj
            });

            // FS
            let res = glam::Vec2::from(ra::size_scaled());
            rg::apply_uniforms_as_bytes(rg::ShaderStage::Fs, 0, &res);

            let time = (Instant::now() - self.start_time).as_secs_f32();
            rg::apply_uniforms_as_bytes(rg::ShaderStage::Fs, 1, &time);

            let mouse = glam::Vec2::new(ra::width() as f32, ra::height() as f32);
            rg::apply_uniforms_as_bytes(rg::ShaderStage::Fs, 2, &mouse);
        }

        self.draw();

        rg::end_pass();
    }

    /// Just draw a fullscreen quad
    fn draw(&mut self) {
        let w = ra::width() as f32;
        let h = ra::height() as f32;

        self.mesh.verts[0].pos = [0.0, 0.0];
        self.mesh.verts[1].pos = [w, 0.0];
        self.mesh.verts[2].pos = [0.0, h];
        self.mesh.verts[3].pos = [w, h];

        unsafe {
            self.mesh.upload_all_verts();
        }
        self.mesh.draw(0, 6);
    }
}
