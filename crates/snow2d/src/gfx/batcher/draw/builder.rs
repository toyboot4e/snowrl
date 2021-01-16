//! Builder for [`QuadParams`]

use crate::gfx::{
    batcher::draw::{DrawApiData, QuadIter, QuadParams, Texture2d},
    geom2d::*,
    Color,
};

// TODO: remove `Scaled` enum (refer to uv_rect of sprite push)

// --------------------------------------------------------------------------------
// traits

/// What can be pushed onto [`QuadParamsBuilder`] by [`SpritePush`]
pub trait OnSpritePush: Texture2d {
    /// Initializes a quad when starting to build a quad
    ///
    /// Note that the quad is initialized to default value before this function is called.
    fn init_quad(&self, builder: &mut impl QuadParamsBuilder);

    #[inline]
    fn push_quad<Q: QuadIter>(&self, draw: &mut DrawApiData<Q>, flips: Flips)
    where
        Self: Sized,
    {
        draw.params
            .write_to_quad(draw.quad_iter.next_quad_mut(self.img()), self, flips);
    }
}

impl QuadParamsBuilder for QuadParams {
    fn params(&mut self) -> &mut QuadParams {
        self
    }
}

/// Comes with default implementation
pub trait QuadParamsBuilder {
    /// Mainly for default implementations, but can be used to modify [`QuadParams`] manually
    fn params(&mut self) -> &mut QuadParams;

    /// Set source rectangle in normalized coordinates ([x, y, w, h])
    ///
    /// You might want to specify destination size, too.
    fn uv_rect(&mut self, rect: impl Into<Rect2f>) -> &mut Self {
        self.params().src_rect = Scaled::Normalized(rect.into());
        self
    }

    /// Set the source rectangle in normalized pixels ([x, y, w, h])
    ///
    /// You might want to specify destination size, too.
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

/// Primary interface to push sprite
#[derive(Debug)]
pub struct SpritePush<'a, 'b, 'c, Q: QuadIter, T: OnSpritePush> {
    draw: DrawApiData<'a, 'b, Q>,
    sprite: &'c T,
    flips: Flips,
}

/// Push sprite to batch data when it goes out of scope
impl<'a, 'b, 'c, Q: QuadIter, T: OnSpritePush> Drop for SpritePush<'a, 'b, 'c, Q, T> {
    fn drop(&mut self) {
        self.sprite.push_quad(&mut self.draw, self.flips);
    }
}

impl<'a, 'b, 'c, Q: QuadIter, T: OnSpritePush> SpritePush<'a, 'b, 'c, Q, T> {
    pub fn new(draw: DrawApiData<'a, 'b, Q>, sprite: &'c T) -> Self {
        draw.params.reset_to_defaults();

        sprite.init_quad(draw.params);

        Self {
            draw,
            sprite,
            flips: Flips::NONE,
        }
    }
}

impl<'a, 'b, 'c, Q: QuadIter, T: OnSpritePush> QuadParamsBuilder for SpritePush<'a, 'b, 'c, Q, T> {
    fn params(&mut self) -> &mut QuadParams {
        &mut self.draw.params
    }
}
