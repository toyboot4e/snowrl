//! Builder for [`RenderPass`]

use rokol::gfx as rg;

use crate::gfx::{tex::RenderTexture, RenderPass, Shader, Snow2d};

const M_INV_Y: glam::Mat4 = glam::const_mat4!(
    [1.0, 0.0, 0.0, 0.0],
    [0.0, -1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0]
);

/// Marker for "typestate" pattern
pub trait RenderPassBuilderState {}

/// Builder for [`RenderPass`]
#[derive(Debug)]
pub struct RenderPassBuilder<'a, 'b, T: RenderPassBuilderState> {
    pub snow: &'a mut Snow2d,
    pub pa: Option<&'b rg::PassAction>,
    pub state: T,
}

impl<'a, 'b, T: RenderPassBuilderState> RenderPassBuilder<'a, 'b, T> {
    pub fn pa(self, pa: Option<&'b rg::PassAction>) -> Self {
        Self {
            snow: self.snow,
            pa,
            state: self.state,
        }
    }
}

#[derive(Debug, Default)]
pub struct ScreenPass<'a> {
    /// Projection = orthographic_projection * transform (tfm)
    ///
    /// transform = view * parent * local
    pub tfm: Option<glam::Mat4>,
    pub shd: Option<&'a Shader>,
}

impl<'a> RenderPassBuilderState for ScreenPass<'a> {}

impl<'a, 'b, 'c> RenderPassBuilder<'a, 'b, ScreenPass<'c>> {
    pub fn new(snow: &'a mut Snow2d) -> Self {
        Self {
            snow,
            pa: None,
            state: ScreenPass {
                tfm: None,
                shd: None,
            },
        }
    }

    pub fn shader(self, shd: Option<&'c Shader>) -> Self {
        Self {
            snow: self.snow,
            pa: self.pa,
            state: ScreenPass {
                tfm: self.state.tfm,
                shd,
            },
        }
    }

    pub fn transform(self, tfm: Option<glam::Mat4>) -> Self {
        Self {
            snow: self.snow,
            pa: self.pa,
            state: ScreenPass {
                tfm,
                shd: self.state.shd,
            },
        }
    }

    pub fn build(self) -> RenderPass<'a> {
        {
            let fbuf = self.snow.window.framebuf_size_u32();
            let pa = self.pa.unwrap_or(&rg::PassAction::LOAD);
            rg::begin_default_pass(pa, fbuf[0], fbuf[1]);
        }

        let shd = self.state.shd.unwrap_or(&self.snow.ons_shd);
        shd.apply_pip();

        // left, right, bottom, top, near, far
        let win_size = self.snow.window.size_f32();
        let mut proj = glam::Mat4::orthographic_rh_gl(0.0, win_size[0], win_size[1], 0.0, 0.0, 1.0);

        if let Some(tfm) = self.state.tfm {
            proj = proj * tfm;
        }

        let bytes: &[u8] = unsafe {
            std::slice::from_raw_parts(
                &proj as *const _ as *const _,
                std::mem::size_of::<glam::Mat4>(),
            )
        };
        shd.set_vs_uniform(0, bytes);

        RenderPass { snow: self.snow }
    }
}

#[derive(Debug)]
pub struct OffscreenPass<'a, 'b> {
    /// Projection = orthographic_projection * transform (tfm)
    ///
    /// transform = view * parent * local
    pub tfm: Option<glam::Mat4>,
    pub shd: Option<&'a Shader>,
    pub target: &'b mut RenderTexture,
}

impl<'a, 'b> RenderPassBuilderState for OffscreenPass<'a, 'b> {}

impl<'a, 'b, 'c, 'd> RenderPassBuilder<'a, 'b, OffscreenPass<'c, 'd>> {
    pub fn new(snow: &'a mut Snow2d, target: &'d mut RenderTexture) -> Self {
        Self {
            snow,
            pa: None,
            state: OffscreenPass {
                tfm: None,
                shd: None,
                target,
            },
        }
    }

    pub fn shader(self, shd: Option<&'c Shader>) -> Self {
        Self {
            snow: self.snow,
            pa: self.pa,
            state: OffscreenPass {
                tfm: self.state.tfm,
                shd,
                target: self.state.target,
            },
        }
    }

    pub fn transform(self, tfm: Option<glam::Mat4>) -> Self {
        Self {
            snow: self.snow,
            pa: self.pa,
            state: OffscreenPass {
                tfm,
                shd: self.state.shd,
                target: self.state.target,
            },
        }
    }

    pub fn build(self) -> RenderPass<'a> {
        {
            let pa = self.pa.unwrap_or(&rg::PassAction::LOAD);
            rg::begin_pass(self.state.target.pass(), pa);
        }

        let shd = self.state.shd.unwrap_or(&self.snow.ons_shd);
        shd.apply_pip();

        // left, right, bottom, top, near, far
        let win_size = self.snow.window.size_f32();
        let mut proj = glam::Mat4::orthographic_rh_gl(0.0, win_size[0], win_size[1], 0.0, 0.0, 1.0);

        // [OpenGL] invert/flip y (TODO: why?)
        proj = M_INV_Y * proj;

        if let Some(tfm) = self.state.tfm {
            proj = proj * tfm;
        }

        let bytes: &[u8] = unsafe {
            std::slice::from_raw_parts(
                &proj as *const _ as *const _,
                std::mem::size_of::<glam::Mat4>(),
            )
        };
        shd.set_vs_uniform(0, bytes);

        RenderPass { snow: self.snow }
    }
}
