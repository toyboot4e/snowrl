/*!

Snow2D

A 2D framework on top of [`rokol`].

*/

pub use rokol;

pub mod gfx;

use rokol::{
    app as ra,
    gfx::{self as rg, BakedResource, Pipeline},
};

use crate::gfx::{
    batcher::{
        draw::*,
        vertex::{QuadData, VertexData},
        Batch,
    },
    tex::RenderTexture,
};

const M_INV_Y: glam::Mat4 = glam::const_mat4!(
    [1.0, 0.0, 0.0, 0.0],
    [0.0, -1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0]
);

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

/// Parameter to [`Snow2d::screen`] or [`Snow2d::offscreen`]
///
/// Shared between on-screen and off-screen rendering pass.
#[derive(Debug)]
pub struct PassConfig<'a> {
    pub pa: &'a rg::PassAction,
    /// uniform matrix = orthographic * transform
    pub tfm: Option<glam::Mat4>,
    pub pip: Option<rg::Pipeline>,
}

/// The 2D renderer
#[derive(Debug, Default)]
pub struct Snow2d {
    /// Vertex/index buffer and images slots
    pub batch: Batch,
    /// Default pipeline object for on-screen rendering
    pub screen_pip: rg::Pipeline,
    /// Default pipeline object for off-screen rendering
    pub ofs_pip: rg::Pipeline,
}

impl Snow2d {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    /// Only be called from [`rokol::app::RApp::init`].
    pub unsafe fn init(&mut self) {
        // create white dot image
        crate::gfx::batcher::draw::init();

        self.batch.init();

        let mut desc = rg::PipelineDesc {
            shader: gfx::shaders::tex_1(),
            index_type: rg::IndexType::UInt16 as u32,
            layout: VertexData::layout_desc(),
            blend: ALPHA_BLEND,
            rasterizer: rg::RasterizerState {
                // NOTE: our renderer may output backward triangle
                cull_mode: rg::CullMode::None as u32,
                ..Default::default()
            },
            ..Default::default()
        };

        self.screen_pip = Pipeline::create(&desc);

        self.ofs_pip = Pipeline::create({
            desc.blend = ALPHA_BLEND;
            desc.blend.depth_format = rg::PixelFormat::Depth as u32;
            // TODO: sample_count? (also on internal image of render texture)
            desc.rasterizer.sample_count = 1;
            &desc
        });
    }

    /// Begins on-screen rendering pass
    pub fn screen(&mut self, cfg: PassConfig<'_>) -> Pass<'_> {
        rg::begin_default_pass(cfg.pa, ra::width(), ra::height());
        rg::apply_pipeline(cfg.pip.unwrap_or(self.screen_pip));

        // left, right, top, bottom, near, far
        let mut proj = glam::Mat4::orthographic_rh_gl(0.0, 1280.0, 720.0, 0.0, 0.0, 1.0);

        if let Some(tfm) = cfg.tfm {
            proj = proj * tfm;
        }

        unsafe {
            rg::apply_uniforms_as_bytes(rg::ShaderStage::Vs, 0, &proj);
        }

        Pass { snow: self }
    }

    /// Begins off-screen rendering pass
    pub fn offscreen(&mut self, ofs: &RenderTexture, cfg: PassConfig<'_>) -> Pass<'_> {
        rg::begin_pass(ofs.pass(), cfg.pa);
        rg::apply_pipeline(cfg.pip.unwrap_or(self.ofs_pip));

        // left, right, top, bottom, near, far
        let mut proj = glam::Mat4::orthographic_rh_gl(0.0, 1280.0, 720.0, 0.0, 0.0, 1.0);

        if let Some(tfm) = cfg.tfm {
            proj = proj * tfm;
        }

        // [OpenGL] invert y
        proj = M_INV_Y * proj;

        unsafe {
            rg::apply_uniforms_as_bytes(rg::ShaderStage::Vs, 0, &proj);
        }

        Pass { snow: self }
    }

    fn end_pass(&mut self) {
        self.batch.flush();
        rg::end_pass();
    }
}

/// [`DrawApi`] for a rendering pass (on-screen or off-screen)
pub struct Pass<'a> {
    snow: &'a mut Snow2d,
}

impl<'a> Drop for Pass<'a> {
    fn drop(&mut self) {
        self.snow.end_pass();
    }
}

impl<'a> DrawApi for Pass<'a> {
    fn _next_quad_mut(&mut self, img: rg::Image) -> &mut QuadData {
        let ix = self.snow.batch.next_quad_ix(img);
        &mut self.snow.batch.mesh.verts[ix]
    }

    fn _next_push_mut(&mut self, tex: &impl Texture2d) -> QuadPush<'_> {
        let target = {
            let ix = self.snow.batch.next_quad_ix(tex.img());
            &mut self.snow.batch.mesh.verts[ix]
        };

        QuadPush {
            params: &mut self.snow.batch.params,
            target,
        }
    }
}
