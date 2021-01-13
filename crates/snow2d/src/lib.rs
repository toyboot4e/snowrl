/*!

Snow2D

A 2D framework on top of [`rokol`].

*/

pub use rokol;

pub mod asset;
pub mod gfx;

use rokol::{
    app as ra,
    fons::FontBook,
    gfx::{self as rg, BakedResource, Pipeline},
};

use crate::gfx::{
    batcher::{
        draw::*,
        vertex::{QuadData, VertexData},
        Batch, BatchData,
    },
    geom2d::*,
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
    src_factor_alpha: rg::BlendFactor::One as u32,
    dst_factor_alpha: rg::BlendFactor::Zero as u32,
    op_alpha: 0,
    color_write_mask: 0,
    color_attachment_count: 1,
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

impl<'a> Default for PassConfig<'a> {
    fn default() -> Self {
        Self {
            pa: &rg::PassAction::NONE,
            tfm: None,
            pip: None,
        }
    }
}

/// The 2D renderer
#[derive(Debug)]
pub struct Snow2d {
    /// Vertex/index buffer and images slots
    pub batch: Batch,
    pub fontbook: FontBook,
    /// Default pipeline object for on-screen rendering
    screen_pip: rg::Pipeline,
    /// Default pipeline object for off-screen rendering
    ofs_pip: rg::Pipeline,
}

impl Snow2d {
    /// Call when rokol is ready
    pub unsafe fn new() -> Self {
        // create white dot image
        crate::gfx::batcher::draw::init();

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

        let screen_pip = Pipeline::create(&desc);

        let ofs_pip = Pipeline::create({
            desc.blend = ALPHA_BLEND;
            desc.blend.depth_format = rg::PixelFormat::Depth as u32;
            // TODO: sample_count? (also on internal image of render texture)
            desc.rasterizer.sample_count = 1;
            &desc
        });

        Self {
            batch: Batch::default(),
            fontbook: FontBook::new(256, 256),
            screen_pip,
            ofs_pip,
        }
    }

    pub fn post_update(&mut self) {
        unsafe {
            self.fontbook.maybe_update_image();
        }
    }

    /// Begins on-screen rendering pass
    pub fn screen(&mut self, cfg: PassConfig<'_>) -> Pass<'_> {
        rg::begin_default_pass(cfg.pa, ra::width(), ra::height());

        // FIXME: pipeline should set uniform by themselves
        rg::apply_pipeline(cfg.pip.unwrap_or(self.screen_pip));

        // left, right, top, bottom, near, far
        let mut proj = glam::Mat4::orthographic_rh_gl(0.0, 1280.0, 720.0, 0.0, 0.0, 1.0);

        if let Some(tfm) = cfg.tfm {
            proj = proj * tfm;
        }

        // FIXME: projection matrix should be set shaders by themselves
        unsafe {
            rg::apply_uniforms_as_bytes(rg::ShaderStage::Vs, 0, &proj);
        }

        Pass { snow: self }
    }

    /// Begins off-screen rendering pass
    pub fn offscreen(&mut self, ofs: &RenderTexture, cfg: PassConfig<'_>) -> Pass<'_> {
        rg::begin_pass(ofs.pass(), cfg.pa);

        // FIXME: pipeline should set uniform by themselves
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
        self.batch.data.flush();
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

impl<'a> QuadIter for Pass<'a> {
    fn peek_quad_mut(&mut self, img: rg::Image) -> &mut QuadData {
        self.snow.batch.data.peek_quad_mut(img)
    }

    fn next_quad_mut(&mut self, img: rg::Image) -> &mut QuadData {
        self.snow.batch.data.next_quad_mut(img)
    }
}

impl<'a> DrawApi for Pass<'a> {
    type Q = BatchData;

    /// Starts a [`QuadParamsBuilder`] setting source/destination size and uv values
    fn sprite<'x, S: OnSpritePush + Texture2d>(
        &mut self,
        sprite: &'x S,
    ) -> SpritePush<'_, '_, 'x, Self::Q, S>
    where
        Self: Sized,
    {
        self.snow.batch.sprite(sprite)
    }
}

impl<'a> Pass<'a> {
    pub fn fontbook(&mut self) -> &mut FontBook {
        &mut self.snow.fontbook
    }

    /// Renders multiple lines of text
    ///
    /// TODO: maybe add it to DrawApi
    pub fn txt(&mut self, pos: impl Into<Vec2f>, text: &str) {
        // use non-premultipiled alpha blending

        // FIXME: fontstash _should_ handle newline.. but not. why?
        // self.render_text_line(text, pos.into());

        // we have to draw text line by line
        let fontsize = 20.0; // really?
        let nl_space = 2.0;

        let pos = pos.into();
        let nl_offset = fontsize + nl_space;
        for (i, line) in text.lines().enumerate() {
            let pos = pos + Vec2f::new(0.0, nl_offset * i as f32);
            self.render_text_line(pos, line);
        }

        // TODO: ensure flushing?
    }

    /// Renders one line of text
    #[inline]
    fn render_text_line(&mut self, pos: Vec2f, text: &str) {
        let img = self.snow.fontbook.img();

        let mut iter = self.snow.fontbook.text_iter(text).unwrap();
        while let Some(quad) = iter.next() {
            let q = self.next_quad_mut(img);

            q[0].uv = [quad.s0, quad.t0];
            q[1].uv = [quad.s1, quad.t0];
            q[2].uv = [quad.s0, quad.t1];
            q[3].uv = [quad.s1, quad.t1];

            q[0].pos = [quad.x0 as f32 + pos.x, quad.y0 as f32 + pos.y];
            q[1].pos = [quad.x1 as f32 + pos.x, quad.y0 as f32 + pos.y];
            q[2].pos = [quad.x0 as f32 + pos.x, quad.y1 as f32 + pos.y];
            q[3].pos = [quad.x1 as f32 + pos.x, quad.y1 as f32 + pos.y];

            let color = [255, 255, 255, 255];
            q[0].color = color;
            q[1].color = color;
            q[2].color = color;
            q[3].color = color;
        }

        // we should update the image because we might have changed it
        self.snow.batch.data.force_set_img(self.snow.fontbook.img());
    }
}
