//! Builder for [`QuadParams`]

use std::marker::PhantomData;

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

// TODO: remove `Scaled` enum (refer to uv_rect of sprite push)

// --------------------------------------------------------------------------------
// traits

/// What can be pushed onto [`QuadParamBuilder`] by [`SpritePush`]
pub trait OnSpritePush {
    /// Internal utility for sprite batching
    fn to_cheat_texture(&self) -> CheatTexture2d;

    /// Initializes a quad when starting to build a quad
    ///
    /// Note that the quad is initialized to default value before this function is called.
    fn init_quad(&self, builder: &mut impl QuadParamsBuilder);

    #[inline]
    fn push_quad(quad: &mut QuadPush<'_>, tex: &CheatTexture2d, flips: Flips) {
        quad.params.write_to_quad(&mut quad.target, tex, flips);
    }
}

/// Comes with default implementation
pub trait QuadParamsBuilder {
    /// Mainly for default implementations, but can be used to modify [`QuadParams`] manually
    fn params(&mut self) -> &mut QuadParams;

    /// Set source rectangle in normalized coordinates
    ///
    /// Specify [x, y] and [w, h].
    fn uv_rect(&mut self, rect: impl Into<Rect2f>) -> &mut Self {
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

    // FIXME: /// Sets the size to the destination
    // fn dst_size_norm(&mut self, ws: impl Into<[f32; 2]>) -> &mut Self {
    //     let ws = ws.into();

    //     let data = self.params();
    //     let mut rect = data.dst_rect.inner().clone();
    //     rect.w = ws[0];
    //     rect.h = ws[1];
    //     data.dst_rect = Scaled::Normalized(rect);

    //     self
    // }

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

    /// FIXME: flips not working
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
pub struct SpritePush<'a, T: OnSpritePush> {
    quad: QuadPush<'a>,
    tex: CheatTexture2d,
    flips: Flips,
    _phantom: PhantomData<T>,
}

/// Push sprite to batch data when it goes out of scope
impl<'a, T: OnSpritePush> Drop for SpritePush<'a, T> {
    fn drop(&mut self) {
        T::push_quad(&mut self.quad, &self.tex, self.flips);
    }
}

impl<'a, T: OnSpritePush> SpritePush<'a, T> {
    pub fn new(mut quad: QuadPush<'a>, sprite: &T) -> Self {
        quad.params.reset_to_defaults();
        sprite.init_quad(&mut quad);

        Self {
            quad,
            tex: sprite.to_cheat_texture(),
            flips: Flips::NONE,
            _phantom: Default::default(),
        }
    }
}

impl<'a, T: OnSpritePush> QuadParamsBuilder for SpritePush<'a, T> {
    fn params(&mut self) -> &mut QuadParams {
        &mut self.quad.params
    }
}
