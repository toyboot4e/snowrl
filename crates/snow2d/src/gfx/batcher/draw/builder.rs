//! Builder for [`QuadParams`]

use rokol::gfx as rg;

use crate::gfx::{
    batcher::{
        draw::{QuadParams, Texture2d},
        vertex::QuadData,
    },
    geom2d::*,
    Color,
};

/// Internal utility for sprite batching
#[derive(Debug, Clone)]
pub struct CheatTexture2d {
    pub img: rg::Image,
    pub w: u32,
    pub h: u32,
}

impl Texture2d for CheatTexture2d {
    fn img(&self) -> rg::Image {
        self.img
    }

    fn w(&self) -> f32 {
        self.w as f32
    }

    fn h(&self) -> f32 {
        self.h as f32
    }
}

// --------------------------------------------------------------------------------
// traits

/// Modifies quad when start to build it
pub trait OnSpritePush {
    /// Internal utility for sprite batching
    fn to_cheat_texture(&self) -> CheatTexture2d;
    /// Sets quad parameters. The quad is initialized before calling this method
    fn on_sprite_push(&self, builder: &mut impl QuadParamsBuilder);
}

/// Texture with size data and region. Used by [`QuadParamsBuilder`]
pub trait SubTexture2d: Texture2d {
    /// [x, y, w, h]: Normalized rectangle that represents a regon in texture
    fn uv_rect(&self) -> [f32; 4];
}

/// Texture with size data, region and other geometry data. Used by [`QuadParamsBuilder`]
pub trait Sprite: SubTexture2d {
    /// Rotation in radian
    fn rot(&self) -> f32;
    fn scale(&self) -> [f32; 2];
    /// Normalized origin
    fn origin(&self) -> [f32; 2];
    fn color(&self) -> Color;
}

/// Comes with default implementation
pub trait QuadParamsBuilder {
    /// Mainly for default implementations, but can be used to modify [`QuadParams`] manually
    fn params(&mut self) -> &mut QuadParams;

    /// Set source rectangle in normalized coordinates
    ///
    /// Specify [x, y] and [w, h].
    fn src_rect_uv(&mut self, rect: impl Into<Rect2f>) -> &mut Self {
        self.params().src_rect = Scaled::Normalized(rect.into());
        self
    }

    /// Set the source rectangle in normalized pixels
    ///
    /// Specify [x, y] and [w, h].
    fn src_rect_px(&mut self, rect: impl Into<Rect2f>) -> &mut Self {
        self.params().src_rect = Scaled::Px(rect.into());
        self
    }

    /// Sets the origin position to the destination
    fn dst_pos_px(&mut self, xs: impl Into<[f32; 2]>) -> &mut Self {
        let xs = xs.into();

        let data = self.params();
        let mut rect = data.dst_rect.inner().clone();
        rect.x = xs[0];
        rect.y = xs[1];
        data.dst_rect = Scaled::Px(rect);

        self
    }

    /// Sets the size to the destination
    fn dst_size_px(&mut self, ws: impl Into<[f32; 2]>) -> &mut Self {
        let ws = ws.into();

        let data = self.params();
        let mut rect = data.dst_rect.inner().clone();
        rect.w = ws[0];
        rect.h = ws[1];
        data.dst_rect = Scaled::Px(rect);

        self
    }

    /// Sets origin position and size to the destination
    fn dst_rect_px(&mut self, xs: impl Into<Rect2f>) -> &mut Self {
        let rect = xs.into();

        let data = self.params();
        data.dst_rect = Scaled::Px(rect.into());

        self
    }

    /// Sets origin where we specify coordinates / where the quad rotates
    fn origin(&mut self, origin: impl Into<Vec2f>) -> &mut Self {
        self.params().origin = origin.into();
        self
    }

    /// Alpha value is considered here, too
    fn color(&mut self, color: Color) -> &mut Self {
        self.params().color = color;
        self
    }

    fn rot(&mut self, rot: f32) -> &mut Self {
        self.params().rot = rot;
        self
    }

    fn flips(&mut self, flips: Flips) -> &mut Self {
        self.params().flips = flips;
        self
    }

    fn skew(&mut self, skew: Skew2f) -> &mut Self {
        self.params().skew = skew;
        self
    }
}

// --------------------------------------------------------------------------------
// [`QuadParamsBuilder`] impls

#[derive(Debug)]
pub struct QuadPush<'a> {
    pub params: &'a mut QuadParams,
    pub target: &'a mut QuadData,
}

impl<'a> QuadParamsBuilder for QuadPush<'a> {
    fn params(&mut self) -> &mut QuadParams {
        &mut self.params
    }
}

/// Primary interface to push sprite
#[derive(Debug)]
pub struct SpritePush<'a> {
    quad: QuadPush<'a>,
    texture: CheatTexture2d,
    flips: Flips,
}

/// Push sprite to batch data when it goes out of scope
impl<'a> Drop for SpritePush<'a> {
    fn drop(&mut self) {
        self.quad
            .params
            .write_to_quad(&mut self.quad.target, &self.texture, self.flips);
    }
}

impl<'a> SpritePush<'a> {
    pub fn new(mut quad: QuadPush<'a>, sprite: &impl OnSpritePush) -> Self {
        quad.params.reset_to_defaults();
        sprite.on_sprite_push(&mut quad);

        Self {
            quad,
            texture: sprite.to_cheat_texture(),
            flips: Flips::NONE,
        }
    }
}

impl<'a> QuadParamsBuilder for SpritePush<'a> {
    fn params(&mut self) -> &mut QuadParams {
        &mut self.quad.params
    }
}
