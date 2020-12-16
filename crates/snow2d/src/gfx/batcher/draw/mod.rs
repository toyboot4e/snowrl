pub mod builder;
pub mod params;

use crate::gfx::{batcher::QuadData, geom2d::*, texture::TextureData2d, Color};

pub use self::{
    builder::*,
    params::{QuadParams, Texture2d},
};

use {once_cell::sync::OnceCell, rokol::gfx as rg};

static WHITE_DOT: OnceCell<TextureData2d> = OnceCell::new();

pub fn init() {
    let dot = include_bytes!("white_dot.png");
    let tex = TextureData2d::from_encoded_bytes(dot).unwrap();
    WHITE_DOT.set(tex).unwrap();
}

/// Quad-based rendering API
pub trait DrawApi {
    /// Modify the quad manually!
    fn _next_quad_mut(&mut self, img: rg::Image) -> &mut QuadData;

    /// Modify the quad manually!
    fn _next_push_mut(&mut self, tex: &impl Texture2d) -> QuadPush<'_>;

    /// (Mainly) internal utilitiy to implement `linep and `rect`
    fn _white_dot(&mut self) -> SpritePush {
        self.sprite(WHITE_DOT.get().unwrap())
    }

    /// Starts a [`QuadParamBuilder`] setting source/destination size and uv values
    ///
    /// [`QuadParamBuilder`]: crate::gfx::batcher::draw::QuadParamBuilder
    fn sprite<S: OnSpritePush + Texture2d>(&mut self, sprite: &S) -> SpritePush {
        // NOTE: the quad is initialized in this method
        SpritePush::new(self._next_push_mut(sprite), sprite)
    }

    fn line(&mut self, p1: impl Into<Vec2f>, p2: impl Into<Vec2f>, color: Color) {
        let p1 = p1.into();
        let p2 = p2.into();

        let delta = p2 - p1;
        let rad = delta.rad();
        let len = delta.len();

        self._white_dot()
            .color(color)
            .dst_rect_px([p1, (len, 1.0).into()])
            .rot(rad);
    }

    fn rect(&mut self, rect: impl Into<Rect2f>, color: Color) {
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