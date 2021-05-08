/*!
Graphics

[`Snow2d`] is the context and the primary API.

# Coordinate system

Same as OpenGL or school math (right-handed and column-major).
*/

pub mod geom2d;
pub mod mesh;
pub mod pass;
pub mod shaders;
pub mod tex;

// immediate-mode rendering
pub mod batch;
pub mod draw;

pub mod text;

pub use pass::RenderPassBuilder;

use std::time::Duration;

use serde::{Deserialize, Serialize};

use rokol::{
    fons::{self as fons, FontTexture},
    gfx::{self as rg, BakedResource},
};

use crate::utils::Inspect;

use self::{
    batch::{Batch, BatchData, QuadData},
    draw::*,
    geom2d::*,
    tex::RenderTexture,
    text::FontBook,
};

/// 4 bytes color data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Inspect)]
#[inspect(as = "[u8; 4]")]
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

    pub fn set_alpha(&mut self, a: u8) {
        self.a = a;
    }
}

macro_rules! def_colors {
    ($($name:ident : $r:expr, $g:expr, $b:expr, $a:expr,)*) => {
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
    OPAQUE:
        255, 255, 255, 255,
    SEMI_TRANSPARENT:
        255, 255, 255, 128,
    TRANSPARENT:
        0, 0, 0, 0,
    WHITE:
        255, 255, 255, 255,
    WHITE_SEMI_TRANSPARENT:
        255, 255, 255, 128,
    WHITE_TRANSPARENT:
        255, 255, 255, 0,
    BLACK:
        0, 0, 0, 0,
    BLACK_SEMI_TRANSPARENT:
        0, 0, 0, 128,
    BLACK_TRANSPARENT:
        0, 0, 0, 0,
    GRAY:
        32, 32, 32, 32,
    CORNFLOWER_BLUE:
        100, 149, 237, 255,
    // TODO: define more colors
);

impl Into<[u8; 4]> for Color {
    fn into(self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

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

/// Shader
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

/// Game time progress (stops while pausing)
#[derive(Debug, Clone)]
pub struct GameClock {
    past: Duration,
}

/// Lifetime
impl GameClock {
    fn new() -> Self {
        Self {
            past: Duration::default(),
        }
    }

    /// Ticks the game time
    pub(crate) fn tick(&mut self, dt: Duration) {
        self.past += dt;
    }
}

impl GameClock {
    /// Past duration (only while the game window is updated)
    pub fn past_duration(&self) -> Duration {
        self.past
    }
}

/// The 2D renderer
#[derive(Debug)]
pub struct Snow2d {
    /// Window state used by the renderer
    pub window: WindowState,
    /// Game time progress
    pub clock: GameClock,
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
            clock: GameClock::new(),
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

    pub fn post_render(&mut self, _dt: Duration) {
        // does nothing..
    }
}

/// API
impl Snow2d {
    /// Returns builder for on-screen rendering pass
    pub fn screen(&mut self) -> RenderPassBuilder<'_, '_, pass::ScreenPass> {
        RenderPassBuilder {
            snow: self,
            pa: None,
            state: pass::ScreenPass {
                tfm: None,
                shd: None,
            },
        }
    }

    /// Returns builder for off-screen rendering pass
    pub fn offscreen<'a>(
        &mut self,
        target: &'a mut RenderTexture,
    ) -> RenderPassBuilder<'_, '_, pass::OffscreenPass<'static, 'a>> {
        RenderPassBuilder {
            snow: self,
            pa: None,
            state: pass::OffscreenPass {
                tfm: None,
                shd: None,
                target,
            },
        }
    }

    fn on_end_pass(&mut self) {
        self.batch.data.flush();
        rg::end_pass();
    }
}

/// Extended [`DrawApi`] for a rendering pass (on-screen or off-screen)
pub struct RenderPass<'a> {
    snow: &'a mut Snow2d,
}

impl<'a> Drop for RenderPass<'a> {
    fn drop(&mut self) {
        self.snow.on_end_pass();
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
