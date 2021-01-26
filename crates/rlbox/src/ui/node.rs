//! OMG UI nodes

use snow2d::gfx::{
    draw::DrawApi,
    geom2d::*,
    tex::{NineSliceSprite, SpriteData},
    Color,
};

use crate::utils::pool::Handle;

#[derive(Debug, Clone)]
pub struct Node {
    pub draw: Draw,
    pub children: Vec<Handle<Self>>,
    /// Drawing order (1.0 is top, 0.0 is bottom)
    pub z: f32,
    /// Geometry data that can be tweened
    pub geom: Geom,
}

impl From<Draw> for Node {
    fn from(draw: Draw) -> Self {
        Node {
            draw,
            children: vec![],
            z: 0.0,
            geom: Geom::default(),
        }
    }
}

impl Node {
    pub fn render(&mut self, draw: &mut impl DrawApi) {
        //
    }
}

/// Common geometry information
#[derive(Debug, Clone, Default)]
pub struct Geom {
    pub pos: Vec2f,
    pub size: Vec2f,
    pub color: Color,
    // /// Rotation in radian
    // pub rot: f32,
    // pub scales: Vec2f,
}

// Everything is drawn as a [`Node`] with a [`Surface`]
#[derive(Debug, Clone)]
pub enum Draw {
    Sprite(SpriteData),
    NineSlice(NineSliceSprite),
}

impl Draw {
    // pub fn draw(
}

/// X/Y aligment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Align2d {
    pub h: AlignH,
    pub v: AlignV,
}

impl Align2d {
    pub fn new(h: AlignH, v: AlignV) -> Self {
        Self { h, v }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignH {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignV {
    Top,
    Center,
    Bottom,
}
