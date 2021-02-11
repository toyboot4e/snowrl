/*!
UI nodes
*/

use snow2d::gfx::{
    draw::*,
    geom2d::*,
    tex::{NineSliceSprite, SpriteData},
    Color,
};

use crate::utils::pool::Handle;

#[derive(Debug, Clone)]
pub struct Node {
    pub draw: Draw,
    /// Geometry data that can be tweened
    pub params: DrawParams,
    pub children: Vec<Handle<Self>>,
    /// Drawing order (1.0 is top, 0.0 is bottom)
    pub z: f32,
}

impl From<Draw> for Node {
    fn from(draw: Draw) -> Self {
        let params = DrawParams {
            size: match draw {
                Draw::Sprite(ref x) => x.sub_tex_size_scaled().into(),

                Draw::NineSlice(ref x) => x.sub_tex_size_scaled().into(),
            },
            ..Default::default()
        };

        Node {
            draw,
            params,
            children: vec![],
            z: 0.0,
        }
    }
}

impl Node {
    pub fn render(&mut self, draw: &mut impl DrawApi) {
        match self.draw {
            Draw::Sprite(ref x) => {
                self.params.build(&mut draw.sprite(x));
            }
            Draw::NineSlice(ref x) => {
                self.params.build(&mut draw.sprite(x));
            }
        }
    }
}

/// Common geometry information to draw a [`Node`]
#[derive(Debug, Clone, Default)]
pub struct DrawParams {
    pub pos: Vec2f,
    pub size: Vec2f,
    pub color: Color,
    // /// Rotation in radian
    // pub rot: f32,
    // pub scales: Vec2f,
}

impl DrawParams {
    pub fn build<'a, 'b: 'a, B: QuadParamsBuilder>(&self, builder: &'b mut B) -> &'a mut B {
        builder
            .dst_pos_px(self.pos)
            .dst_size_px(self.size)
            .color(self.color)
    }
}

// Everything is drawn as a [`Node`] with [`Draw`]
#[derive(Debug, Clone)]
pub enum Draw {
    Sprite(SpriteData),
    NineSlice(NineSliceSprite),
}

impl Draw {
    // pub fn draw(
}

pub struct Text {
    text: String,
    // TODO: decoration information (spans for colors, etc)
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
