/*!

Graphics

# Coordinate system

Same as OpenGL or school math (left-handed and column-major).

*/

pub mod geom2d;
pub mod mesh;
pub mod shaders;
pub mod tex;

// immediate-mode rendering
pub mod batch;
pub mod draw;

pub mod text;

use rokol::{
    app as ra,
    fons::{FonsQuad, FontBook},
    gfx::{self as rg, BakedResource},
};

use self::{
    batch::{Batch, BatchData, QuadData},
    draw::*,
    geom2d::*,
    tex::RenderTexture,
};

/// 4 bytes color data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Default for Color {
    fn default() -> Self {
        Self::WHITE
    }
}

impl Color {
    pub fn to_array(&self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }

    pub fn to_normalized_array(&self) -> [f32; 4] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        ]
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Example: `Color::WHITE.with_alpha(128)`
    pub fn with_alpha(self, a: u8) -> Self {
        let mut x = self;
        x.a = a;
        x
    }
}

macro_rules! def_colors {
    ($(($name:ident, $r:expr, $g:expr, $b:expr, $a:expr)), * $(,)?) => {
        impl Color {
            $(
                pub const $name: Self = Self {
                    r: $r,
                    g: $g,
                    b: $b,
                    a: $a,
                };
            )*
        }
    };
}

def_colors!(
    // name, r, g, b, a
    (OPAQUE, 255, 255, 255, 255),
    (SEMI_TRANSPARENT, 255, 255, 255, 128),
    (TRANSPARENT, 0, 0, 0, 0),
    (WHITE, 255, 255, 255, 255),
    (WHITE_SEMI_TRANSPARENT, 255, 255, 255, 128),
    (WHITE_TRANSPARENT, 255, 255, 255, 0),
    (BLACK, 0, 0, 0, 0),
    (BLACK_SEMI_TRANSPARENT, 0, 0, 0, 128),
    (BLACK_TRANSPARENT, 0, 0, 0, 0),
    (GRAY, 32, 32, 32, 32),
    (CORNFLOWER_BLUE, 100, 149, 237, 255),
    // TODO: define more colors
);

impl From<[u8; 4]> for Color {
    fn from(xs: [u8; 4]) -> Self {
        Self::rgba(xs[0], xs[1], xs[2], xs[3])
    }
}

impl From<&[u8; 4]> for Color {
    fn from(xs: &[u8; 4]) -> Self {
        Self::rgba(xs[0], xs[1], xs[2], xs[3])
    }
}

impl From<[u8; 3]> for Color {
    fn from(xs: [u8; 3]) -> Self {
        Self::rgb(xs[0], xs[1], xs[2])
    }
}

impl From<&[u8; 3]> for Color {
    fn from(xs: &[u8; 3]) -> Self {
        Self::rgb(xs[0], xs[1], xs[2])
    }
}

#[derive(Debug)]
pub struct Shader {
    pub shd: rg::Shader,
    pub pip: rg::Pipeline,
}

impl std::ops::Drop for Shader {
    fn drop(&mut self) {
        rg::Shader::destroy(self.shd);
        rg::Pipeline::destroy(self.pip);
    }
}

impl Shader {
    pub fn new(shd: rg::Shader, pip: rg::Pipeline) -> Self {
        Self { shd, pip }
    }

    pub fn set_vs_uniform(&self, ix: usize, bytes: &[u8]) {
        rg::apply_uniforms(rg::ShaderStage::Vs, ix as u32, bytes);
    }

    pub fn set_fs_uniform(&self, ix: usize, bytes: &[u8]) {
        rg::apply_uniforms(rg::ShaderStage::Fs, ix as u32, bytes);
    }

    pub fn apply_pip(&self) {
        rg::apply_pipeline(self.pip);
    }
}

// TODO: define operators
const M_INV_Y: glam::Mat4 = glam::const_mat4!(
    [1.0, 0.0, 0.0, 0.0],
    [0.0, -1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0]
);

/// Parameter to [`Snow2d::screen`] or [`Snow2d::offscreen`]
///
/// Shared between on-screen and off-screen rendering pass.
#[derive(Debug)]
pub struct PassConfig<'a> {
    pub pa: &'a rg::PassAction,
    /// uniform matrix = orthographic * transform (transform = view)
    pub tfm: Option<glam::Mat4>,
    pub shd: Option<&'a Shader>,
}

impl<'a> Default for PassConfig<'a> {
    fn default() -> Self {
        Self {
            pa: &rg::PassAction::LOAD,
            tfm: None,
            shd: None,
        }
    }
}

/// The 2D renderer
#[derive(Debug)]
pub struct Snow2d {
    /// Vertex/index buffer and images slots
    batch: Batch,
    pub fontbook: FontBook,
    /// Shader program for on-screen rendering
    ons_shd: Shader,
    /// Shader program for off-screen rendering
    ofs_shd: Shader,
}

impl Snow2d {
    /// Call when rokol is ready
    pub unsafe fn new() -> Self {
        // create white dot image
        crate::gfx::draw::init();

        Self {
            batch: Batch::default(),
            fontbook: FontBook::new(256, 256),
            ons_shd: shaders::default_screen(),
            ofs_shd: shaders::default_offscreen(),
        }
    }

    pub fn pre_render(&mut self) {
        // probablly we measure text before rendendering, so
        // this is the proper place to update GPU texture with CPU texture
        unsafe {
            // call it every frame but only once
            self.fontbook.maybe_update_image();
        }
    }

    /// Begins on-screen rendering pass
    pub fn screen(&mut self, cfg: PassConfig<'_>) -> RenderPass<'_> {
        rg::begin_default_pass(cfg.pa, ra::width(), ra::height());

        let shd = cfg.shd.unwrap_or(&self.ons_shd);
        shd.apply_pip();

        // FIXME: projection matrix should be set shaders by themselves
        // left, right, bottom, top, near, far
        // let mut proj = glam::Mat4::orthographic_rh_gl(0.0, 1280.0, 720.0, 0.0, 0.0, 1.0);
        let mut proj = glam::Mat4::orthographic_rh_gl(0.0, 1280.0, 720.0, 0.0, 0.0, 1.0);

        if let Some(tfm) = cfg.tfm {
            proj = proj * tfm;
        }

        let bytes: &[u8] = unsafe {
            std::slice::from_raw_parts(
                &proj as *const _ as *const _,
                std::mem::size_of::<glam::Mat4>(),
            )
        };
        shd.set_vs_uniform(0, bytes);

        RenderPass { snow: self }
    }

    /// Begins off-screen rendering pass
    pub fn offscreen(&mut self, ofs: &mut RenderTexture, cfg: PassConfig<'_>) -> RenderPass<'_> {
        rg::begin_pass(ofs.pass(), cfg.pa);

        // we don't need mutability for `ofs` actually
        let shd = cfg.shd.unwrap_or(&self.ofs_shd);
        shd.apply_pip();

        // FIXME: projection matrix should be set shaders by themselves
        // left, right, bottom, top, near, far
        let mut proj = glam::Mat4::orthographic_rh_gl(0.0, 1280.0, 720.0, 0.0, 0.0, 1.0);

        if let Some(tfm) = cfg.tfm {
            proj = proj * tfm;
        }

        // [OpenGL] invert/flip y (TODO: why?)
        proj = M_INV_Y * proj;

        let bytes: &[u8] = unsafe {
            std::slice::from_raw_parts(
                &proj as *const _ as *const _,
                std::mem::size_of::<glam::Mat4>(),
            )
        };
        shd.set_vs_uniform(0, bytes);

        RenderPass { snow: self }
    }

    fn end_pass(&mut self) {
        self.batch.data.flush();
        rg::end_pass();
    }
}

/// [`DrawApi`] for a rendering pass (on-screen or off-screen)
pub struct RenderPass<'a> {
    snow: &'a mut Snow2d,
}

impl<'a> Drop for RenderPass<'a> {
    fn drop(&mut self) {
        self.snow.end_pass();
    }
}

impl<'a> QuadIter for RenderPass<'a> {
    fn peek_quad_mut(&mut self, img: rg::Image) -> &mut QuadData {
        self.snow.batch.data.peek_quad_mut(img)
    }

    fn next_quad_mut(&mut self, img: rg::Image) -> &mut QuadData {
        self.snow.batch.data.next_quad_mut(img)
    }
}

impl<'a> DrawApi for RenderPass<'a> {
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

impl<'a> RenderPass<'a> {
    pub fn fontbook(&mut self) -> &mut FontBook {
        &mut self.snow.fontbook
    }

    // pub fn txt(&mut self, pos: impl Into<Vec2f>, text: &str) {

    /// Renders multiple lines of text
    pub fn text(&mut self, pos: impl Into<Vec2f>, text: &str) {
        // NOTE: Be sure to use non-premultiplied alpha blending

        // we have to draw text line by line
        let fontsize = 20.0; // really?
                             // TODO: read configuration
        let nl_space = 2.0;

        let pos = pos.into();
        let nl_offset = fontsize + nl_space;
        for (i, line) in text.lines().enumerate() {
            let pos = pos + Vec2f::new(0.0, nl_offset * i as f32);
            self.line_with_shadow(line, pos, Vec2f::new(2.0, 2.0));
        }
    }

    /// * `base_pos`: left-up corner of text bounds
    #[inline]
    fn line(&mut self, text: &str, base_pos: Vec2f) {
        let img = self.snow.fontbook.img();

        let iter = self.snow.fontbook.text_iter(text).unwrap();
        for fons_quad in iter {
            let q = self.next_quad_mut(img);
            Self::set_quad(q, &fons_quad, base_pos, [255, 255, 255, 255]);
        }
    }

    /// * `base_pos`: left-up corner of text bounds
    #[inline]
    fn line_with_shadow(&mut self, text: &str, base_pos: Vec2f, shadow_offset: Vec2f) {
        let img = self.snow.fontbook.img();
        let shadow_pos = base_pos.offset(shadow_offset);

        let iter = self.snow.fontbook.text_iter(text).unwrap();
        for fons_quad in iter {
            let q_shadow = self.next_quad_mut(img);
            Self::set_quad(q_shadow, &fons_quad, shadow_pos, [0, 0, 0, 255]);

            let q = self.next_quad_mut(img);
            Self::set_quad(q, &fons_quad, base_pos, [255, 255, 255, 255]);
        }
    }

    /// * `base_pos`: left-up corner of text bounds
    #[inline]
    fn line_with_multi_shadow(&mut self, text: &str, base_pos: Vec2f, n_shadows: usize) {
        let img = self.snow.fontbook.img();

        let iter = self.snow.fontbook.text_iter(text).unwrap();
        for fons_quad in iter {
            for i in 0..n_shadows {
                let q_shadow = self.next_quad_mut(img);
                Self::set_quad(
                    q_shadow,
                    &fons_quad,
                    base_pos.offset((i + 1) as f32 * Vec2f::new(1.0, 1.0)),
                    [0, 0, 0, 255],
                );
            }

            let q = self.next_quad_mut(img);
            Self::set_quad(q, &fons_quad, base_pos, [255, 255, 255, 255]);
        }
    }

    #[inline]
    fn set_quad(q: &mut QuadData, fons_quad: &FonsQuad, base_pos: Vec2f, color: [u8; 4]) {
        q[0].uv = [fons_quad.s0, fons_quad.t0];
        q[1].uv = [fons_quad.s1, fons_quad.t0];
        q[2].uv = [fons_quad.s0, fons_quad.t1];
        q[3].uv = [fons_quad.s1, fons_quad.t1];

        q[0].pos = [
            fons_quad.x0 as f32 + base_pos.x,
            fons_quad.y0 as f32 + base_pos.y,
        ];
        q[1].pos = [
            fons_quad.x1 as f32 + base_pos.x,
            fons_quad.y0 as f32 + base_pos.y,
        ];
        q[2].pos = [
            fons_quad.x0 as f32 + base_pos.x,
            fons_quad.y1 as f32 + base_pos.y,
        ];
        q[3].pos = [
            fons_quad.x1 as f32 + base_pos.x,
            fons_quad.y1 as f32 + base_pos.y,
        ];

        q[0].color = color;
        q[1].color = color;
        q[2].color = color;
        q[3].color = color;
    }
}
