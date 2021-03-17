/*!
Graphics module built

# Coordinate system

Same as OpenGL or school math (right-handed and column-major).
*/

pub mod geom2d;
pub mod mesh;
pub mod shaders;
pub mod tex;

// immediate-mode rendering
pub mod batch;
pub mod draw;

pub mod text;

use serde::{Deserialize, Serialize};

use rokol::{
    fons::{self as fons, FontTexture},
    gfx::{self as rg, BakedResource},
};

use self::{
    batch::{Batch, BatchData, QuadData},
    draw::*,
    geom2d::*,
    tex::RenderTexture,
    text::FontBook,
};

/// 4 bytes color data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

/// Platfrom-independent window state representation
#[derive(Debug, Clone)]
pub struct WindowState {
    pub w: u32,
    pub h: u32,
    /// (w, h) * dpi_scale = frame_buffer_size
    pub dpi_scale: [f32; 2],
}

impl WindowState {
    pub fn size_u32(&self) -> [u32; 2] {
        [self.w as u32, self.h as u32]
    }

    pub fn size_f32(&self) -> [f32; 2] {
        [self.w as f32, self.h as f32]
    }

    pub fn framebuf_size_usize(&self) -> [usize; 2] {
        [
            (self.w as f32 * self.dpi_scale[0]) as usize,
            (self.h as f32 * self.dpi_scale[1]) as usize,
        ]
    }

    pub fn framebuf_size_u32(&self) -> [u32; 2] {
        [
            (self.w as f32 * self.dpi_scale[0]) as u32,
            (self.h as f32 * self.dpi_scale[1]) as u32,
        ]
    }

    pub fn framebuf_size_f32(&self) -> [f32; 2] {
        [
            self.w as f32 * self.dpi_scale[0],
            self.h as f32 * self.dpi_scale[1],
        ]
    }
}

/// The 2D renderer
#[derive(Debug)]
pub struct Snow2d {
    /// Window state used by the renderer
    pub window: WindowState,
    /// Vertex/index buffer and images slots
    pub batch: Batch,
    pub fontbook: FontBook,
    /// Shader program for on-screen rendering
    ons_shd: Shader,
    /// Shader program for off-screen rendering
    ofs_shd: Shader,
}

/// Lifeycle
impl Snow2d {
    /// Call when rokol is ready
    pub unsafe fn new(window: WindowState) -> Self {
        // create white dot image
        crate::gfx::draw::init();

        Self {
            window,
            batch: Batch::default(),
            fontbook: {
                let fontbook = FontBook {
                    tex: FontTexture::new(256, 256),
                    storage: Default::default(),
                };
                fontbook.tex.set_align(fons::Align::TOP | fons::Align::LEFT);
                fontbook
            },
            ons_shd: shaders::default_screen(),
            ofs_shd: shaders::default_offscreen(),
        }
    }

    pub fn pre_render(&mut self, window: WindowState) {
        // FIXME: frame buffer size?
        // FIXME: on window size change
        self.window = window;

        // probablly we measure text before rendendering, so
        // this is the proper place to update GPU texture with CPU texture
        unsafe {
            // call it every frame but only once
            self.fontbook.tex.maybe_update_image();
        }
    }
}

/// API
impl Snow2d {
    /// Begins on-screen rendering pass
    pub fn screen(&mut self, cfg: PassConfig<'_>) -> RenderPass<'_> {
        {
            // let fbuf = self.window.framebuf_size_u32();
            let fbuf = self.window.size_u32();
            rg::begin_default_pass(cfg.pa, fbuf[0], fbuf[1]);
        }

        let shd = cfg.shd.unwrap_or(&self.ons_shd);
        shd.apply_pip();

        // FIXME: projection matrix should be set shaders by themselves
        // left, right, bottom, top, near, far
        let win_size = self.window.size_f32();
        let mut proj = glam::Mat4::orthographic_rh_gl(0.0, win_size[0], win_size[1], 0.0, 0.0, 1.0);

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
        let win_size = [self.window.w as f32, self.window.h as f32];
        let mut proj = glam::Mat4::orthographic_rh_gl(0.0, win_size[0], win_size[1], 0.0, 0.0, 1.0);

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

        if let Some(shadow_offset) = Some(Vec2f::new(2.0, 2.0)) {
            for (i, line) in text.lines().enumerate() {
                let pos = pos + Vec2f::new(0.0, nl_offset * i as f32);
                self.text_line_with_shadow(line, pos, shadow_offset);
            }
        } else {
            for (i, line) in text.lines().enumerate() {
                let pos = pos + Vec2f::new(0.0, nl_offset * i as f32);
                self.text_line(line, pos);
            }
        }
    }

    /// * `base_pos`: left-up corner of text bounds
    #[inline]
    fn text_line(&mut self, text: &str, base_pos: Vec2f) {
        let img = self.snow.fontbook.tex.img();

        let iter = self.snow.fontbook.tex.text_iter(text).unwrap();
        for fons_quad in iter {
            let q = self.next_quad_mut(img);
            crate::gfx::text::set_text_quad(q, &fons_quad, base_pos, [255, 255, 255, 255]);
        }
    }

    /// * `base_pos`: left-up corner of text bounds
    #[inline]
    fn text_line_with_shadow(&mut self, text: &str, base_pos: Vec2f, shadow_offset: Vec2f) {
        let img = self.snow.fontbook.tex.img();

        let iter = self.snow.fontbook.tex.text_iter(text).unwrap();
        for fons_quad in iter {
            crate::gfx::text::set_text_quad_with_shadow(
                self,
                img,
                &fons_quad,
                base_pos,
                [255, 255, 255, 255],
                shadow_offset,
                [0, 0, 0, 255],
            );
        }
    }
}
