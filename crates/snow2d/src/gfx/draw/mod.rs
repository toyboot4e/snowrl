//! Fluent drawing API based on quad batcher

mod builder;
mod params;

pub use self::{
    builder::{OnSpritePush, QuadParamsBuilder, SpritePush},
    params::{QuadParams, Texture2d},
};

use {once_cell::sync::OnceCell, rokol::gfx as rg};

use crate::gfx::{batch::QuadData, geom2d::*, tex::Texture2dDrop, Color};

static WHITE_DOT: OnceCell<Texture2dDrop> = OnceCell::new();

pub(crate) fn init() {
    let dot = include_bytes!("white_dot.png");
    let tex = Texture2dDrop::from_encoded_bytes(dot).unwrap();
    WHITE_DOT.set(tex).unwrap();
}

pub trait QuadIter {
    /// Used for implementing the provided methods
    fn peek_quad_mut(&mut self, img: rg::Image) -> &mut QuadData;

    /// Used for implementing the provided methods
    fn next_quad_mut(&mut self, img: rg::Image) -> &mut QuadData;
}

/// Internal binding for implementing quad-based rendering
#[derive(Debug)]
pub struct DrawApiData<'a, 'b, Q: QuadIter> {
    pub quad_iter: &'a mut Q,
    pub params: &'b mut QuadParams,
}

/// Quad-based rendering API on [`QuadIter`]
pub trait DrawApi: QuadIter {
    type Q: QuadIter;

    /// Starts a [`QuadParamsBuilder`] setting source/destination size and uv values
    fn sprite<'a, S: OnSpritePush + Texture2d>(
        &mut self,
        sprite: &'a S,
    ) -> SpritePush<'_, '_, 'a, Self::Q, S>
    where
        Self: Sized;

    /// Used for implementing the provided methods
    fn white_dot(&mut self) -> SpritePush<Self::Q, Texture2dDrop>
    where
        Self: Sized,
    {
        self.sprite(WHITE_DOT.get().unwrap())
    }

    fn line(&mut self, p1: impl Into<Vec2f>, p2: impl Into<Vec2f>, color: Color)
    where
        Self: Sized,
    {
        let p1 = p1.into();
        let p2 = p2.into();

        let delta = p2 - p1;
        let rad = delta.rad();
        let len = delta.len();

        self.white_dot()
            .color(color)
            .dst_rect_px([p1, (len, 1.0).into()])
            .rot(rad);
    }

    fn rect(&mut self, rect: impl Into<Rect2f>, color: Color)
    where
        Self: Sized,
    {
        let rect = rect.into();
        let (p1, p2, p3, p4) = (
            rect.left_up(),
            rect.right_up(),
            rect.right_down(),
            rect.left_down(),
        );

        self.line(p1, p2, color);
        self.line(p2, p3, color);
        self.line(p3, p4, color);
        // FIXME: allow p4 -> p1
        self.line(p1, p4, color);
    }
}
