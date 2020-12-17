//! [`QuadParams`] to write to [`QuadData`]

use rokol::gfx as rg;

use crate::gfx::{batcher::vertex::QuadData, geom2d::*, Color};

/// Texture with size data. Used by [`QuadParams`]
pub trait Texture2d {
    fn img(&self) -> rg::Image;
    /// Pixels
    fn w(&self) -> f32;
    /// Pixels
    fn h(&self) -> f32;
}

/// Full-featured geometry parameters to push a quadliteral onto [`SpriteBatch`]
#[derive(Debug, Clone)]
pub struct QuadParams {
    // TODO: consider using two vectors per src/dest
    pub src_rect: Scaled<Rect2f>,
    pub dst_rect: Scaled<Rect2f>,
    /// Normalized origin
    pub origin: Vec2f,
    pub color: Color,
    pub rot: f32,
    pub flips: Flips,
    pub skew: Skew2f,
}

impl Default for QuadParams {
    fn default() -> Self {
        Self {
            src_rect: Scaled::Normalized(Rect2f::unit()),
            dst_rect: Scaled::Normalized(Rect2f::default()),
            origin: Vec2f::default(),
            color: Color::WHITE,
            rot: 0.0,
            flips: Flips::NONE,
            skew: Skew2f::default(),
        }
    }
}

impl QuadParams {
    pub fn reset_to_defaults(&mut self) {
        // TODO: idionmatically?
        self.src_rect = Scaled::Normalized(Rect2f::unit());
        self.dst_rect = Scaled::Normalized(Rect2f::default());
        self.origin = Vec2f::default();
        self.color = Color::WHITE;
        self.rot = 0.0;
        self.flips = Flips::NONE;
        self.skew = Skew2f::default();
    }

    /// Be sure to flush [`SpriteBatch`] before running if it's saturated.
    pub fn write_to_quad(&self, quad: &mut QuadData, texture: &impl Texture2d, flips: Flips) {
        let (src_rect, dst_rect) = self.geometry_normalized(texture);

        // TODO: round
        // if policy.do_round {
        // dst_rect.x = dst_rect.x.round();
        // dst_rect.y = dst_rect.y.round();
        // }

        self::set_quad(
            quad,
            self.skew,
            self.origin,
            src_rect,
            dst_rect,
            self.color,
            self.rot,
            flips,
        );
    }

    /// -> (src_rect, origin, dst_rect)
    #[inline]
    fn geometry_normalized(&self, texture: &impl Texture2d) -> (Rect2f, Rect2f) {
        let inv_tex_w = 1.0 / texture.w();
        let inv_tex_h = 1.0 / texture.h();

        // in uvs
        let src_rect = match &self.src_rect {
            Scaled::Normalized(uvs) => uvs.clone(),
            Scaled::Px(rect) => Rect2f {
                x: rect.x * inv_tex_w,
                y: rect.y * inv_tex_h,
                w: rect.w * inv_tex_w,
                h: rect.h * inv_tex_h,
            },
        };

        // in pixel
        let dst_rect = match &self.dst_rect {
            Scaled::Normalized(rect) => Rect2f {
                x: rect.x * texture.w(),
                y: rect.y * texture.h(),
                w: rect.w * texture.w(),
                h: rect.h * texture.h(),
            },
            Scaled::Px(rect) => Rect2f {
                x: rect.x,
                y: rect.y,
                w: rect.w,
                h: rect.h,
            },
        };

        // TODO: round
        // dst_rect.x = dst_rect.x.round();
        // dst_rect.y = dst_rect.y.round();

        (src_rect, dst_rect)
    }
}

// --------------------------------------------------------------------------------
// Core

/// Normalized x offsets at top-left, top-right, bottom-left, bottom-right
const CORNER_OFFSET_X: [f32; 4] = [0.0, 1.0, 0.0, 1.0];

/// Normalized y offsets at top-left, top-right, bottom-left, bottom-right
const CORNER_OFFSET_Y: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

/// Pass normalized geometry values
#[inline]
fn set_quad(
    quad: &mut QuadData,
    mut skew: Skew2f,
    origin: Vec2f,
    src_rect: Rect2f,
    dst_rect: Rect2f,
    color: Color,
    rot: f32,
    flips: Flips,
) {
    let rot = Rot2f::from_rad(rot);

    // flip our skew values if we have a flipped sprite
    // FIXME is this OK??
    if flips != Flips::NONE {
        skew.y1 *= -1.0;
        skew.y2 *= -1.0;
        skew.x1 *= -1.0;
        skew.x2 *= -1.0;
    }

    // top, top, bottom, bottom
    let skew_xs = [skew.x1, skew.x1, skew.x2, skew.x2];
    // left, right, right, left
    let skew_ys = [skew.y1, skew.y2, skew.y1, skew.y2];

    // push four vertices: top-left, top-right, bottom-left, and bottom-right, respectively
    for i in 0..4 {
        let corner_x = (CORNER_OFFSET_X[i] - origin.x) * dst_rect.w + skew_xs[i];
        let corner_y = (CORNER_OFFSET_Y[i] - origin.y) * dst_rect.h - skew_ys[i];

        quad[i].pos[0] = (rot.x2 * corner_y) + (rot.x1 * corner_x) + dst_rect.x;
        quad[i].pos[1] = (rot.y2 * corner_y) + (rot.y1 * corner_x) + dst_rect.y;

        // Here, `^` is xor (exclusive or) operator. So if `effects` (actually flips?) equals to
        // zero, it does nothing and `i ^ effects` == `i`
        quad[i].uv[0] = (CORNER_OFFSET_X[i ^ flips.bits() as usize] * src_rect.w) + src_rect.x;
        quad[i].uv[1] = (CORNER_OFFSET_Y[i ^ flips.bits() as usize] * src_rect.h) + src_rect.y;

        quad[i].color = color.to_array();
    }
}
