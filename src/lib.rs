pub mod gfx;

use {
    glam::{Mat3, Vec2},
    rokol::{
        app as ra,
        gfx::{self as rg, BakedResource, Buffer, Pipeline},
    },
    std::path::PathBuf,
};

use crate::gfx::{batcher::Batch, texture::TextureData2d};

/// The 2D renderer on top of [`rokol`]
#[derive(Debug, Default)]
pub struct Snow2d {
    /// Clears the frame color buffer on starting screen rendering pass
    pub pa: rg::PassAction,
    /// Vertex layouts, shader and render states
    pub pip: rg::Pipeline,
    /// Vertex/index buffer and images slots
    pub batch: Batch,
}

impl Snow2d {
    pub fn new() -> Self {
        Self {
            pa: rg::PassAction::clear(gfx::Color::CORNFLOWER_BLUE.to_normalized_array()),
            ..Default::default()
        }
    }

    /// Only be called from [`rokol::app::RApp::init`].
    pub unsafe fn init(&mut self) {
        // create whitee dot image
        crate::gfx::batcher::draw::init();

        self.batch.init();

        self.pip = Pipeline::create(&rg::PipelineDesc {
            shader: gfx::shaders::tex_1(),
            index_type: rg::IndexType::UInt16 as u32,
            layout: {
                let mut desc = rg::LayoutDesc::default();
                desc.attrs[0].format = rg::VertexFormat::Float2 as u32;
                desc.attrs[1].format = rg::VertexFormat::UByte4N as u32;
                desc.attrs[2].format = rg::VertexFormat::Float2 as u32;
                desc
            },
            ..Default::default()
        });
    }

    pub fn begin_default_pass(&mut self) {
        rg::begin_default_pass(&self.pa, ra::width(), ra::height());
        // TODO: use batcher.begin
        rg::apply_pipeline(self.pip);

        // left, right, top, bottom, near, far
        let proj = glam::Mat4::orthographic_rh_gl(0.0, 1280.0, 720.0, 0.0, 0.0, 1.0);
        unsafe {
            rg::apply_uniforms_as_bytes(rg::ShaderStage::Vs, 0, &proj);
        }
    }

    // TODO: begin_pass (PassConfig) then push shader

    // TODO: pop automatically
    pub fn end_pass(&mut self) {
        self.batch.flush();
        // pop shader
        rg::end_pass();
    }
}
