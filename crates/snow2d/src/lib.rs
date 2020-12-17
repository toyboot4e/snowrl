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
    batcher::{draw::*, vertex::QuadData, Batch},
    tex::Texture2dDrop,
};

#[derive(Debug)]
pub struct PassConfig<'a, 'b> {
    pub pa: &'a rg::PassAction,
    // tfm: Option<glam::Mat3>,
    // pip: Option<rg::Pipeline>,
    pub ofs: Option<&'b OffscreenPass>,
}

#[derive(Debug, Default)]
pub struct OffscreenPass {
    /// Render target texture binded to the internal [`rg::Pass`]
    tex: Texture2dDrop,
    pass: rg::Pass,
}

impl OffscreenPass {
    pub fn new(w: u32, h: u32) -> Self {
        let tex = Texture2dDrop::offscreen(w, h);

        let pass = rg::Pass::create(&{
            let mut desc = rg::PassDesc::default();

            desc.color_attachments[0] = rg::AttachmentDesc {
                image: tex.img(),
                mip_level: 0,
                ..Default::default()
            };

            desc.depth_stencil_attachment = rg::AttachmentDesc {
                // FIXME: share creation with texture
                image: rg::Image::create(&{
                    let mut desc = crate::gfx::tex::target_desc(w, h);
                    desc.pixel_format = rg::PixelFormat::Depth as u32;
                    desc
                }),
                ..Default::default()
            };
            desc
        });

        Self { tex, pass }
    }

    pub fn tex(&self) -> &Texture2dDrop {
        &self.tex
    }

    pub fn img(&self) -> rg::Image {
        self.tex.img()
    }
}

/// The 2D renderer
#[derive(Debug, Default)]
pub struct Snow2d {
    pub frame_pip: rg::Pipeline,
    pub ofs_pip: rg::Pipeline,
    /// Vertex/index buffer and images slots
    pub batch: Batch,
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
            layout: {
                let mut desc = rg::LayoutDesc::default();
                desc.attrs[0].format = rg::VertexFormat::Float2 as u32;
                desc.attrs[1].format = rg::VertexFormat::UByte4N as u32;
                desc.attrs[2].format = rg::VertexFormat::Float2 as u32;
                desc
            },
            blend: rg::BlendState {
                enabled: true,
                src_factor_rgb: rg::BlendFactor::SrcAlpha as u32,
                dst_factor_rgb: rg::BlendFactor::OneMinusSrcAlpha as u32,
                ..Default::default()
            },
            ..Default::default()
        };

        self.frame_pip = Pipeline::create(&desc);

        desc.blend.depth_format = rg::PixelFormat::Depth as u32;
        self.ofs_pip = Pipeline::create(&desc);
    }

    pub fn begin_pass(&mut self, cfg: PassConfig) -> Pass<'_> {
        if let Some(ofs) = cfg.ofs {
            // TODO: invert for OpenGL?
            rg::begin_pass(ofs.pass, cfg.pa);
            // TODO: apply given pipeline
            rg::apply_pipeline(self.ofs_pip);
        } else {
            rg::begin_default_pass(cfg.pa, ra::width(), ra::height());
            // TODO: apply given pipeline
            rg::apply_pipeline(self.frame_pip);
        }

        // left, right, top, bottom, near, far
        let proj = glam::Mat4::orthographic_rh_gl(0.0, 1280.0, 720.0, 0.0, 0.0, 1.0);
        // TODO: apply given matrix
        unsafe {
            rg::apply_uniforms_as_bytes(rg::ShaderStage::Vs, 0, &proj);
        }

        Pass { snow: self }
    }

    // TODO: pop automatically
    fn end_pass(&mut self) {
        self.batch.flush();
        // TODO: pop shader if pushed
        rg::end_pass();
    }
}

/// [`DrawApi`] and lifetime to a rendering pass
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
