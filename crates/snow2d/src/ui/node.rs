/*!
UI nodes

[`Handle`] of UI nodes are strong references, so a node won't freed until nothing refers to it.
*/

use crate::{
    gfx::{draw::*, geom2d::*, Color, RenderPass},
    utils::pool::Handle,
};

// Re-exported as [`Node`] variants
pub use crate::gfx::tex::{NineSliceSprite, SpriteData};

/// Visible object in a UI layer
#[derive(Debug, Clone)]
pub struct Node {
    pub draw: Draw,
    /// Common geometry data
    pub params: DrawParams,
    /// TODO: Drawing order (1.0 is top, 0.0 is bottom)
    pub z: f32,
    /// TODO: transform tree
    pub children: Vec<Handle<Self>>,
}

impl From<Draw> for Node {
    fn from(draw: Draw) -> Self {
        let params = DrawParams {
            size: match draw {
                Draw::Sprite(ref x) => x.sub_tex_size_scaled().into(),

                Draw::NineSlice(ref x) => x.sub_tex_size_scaled().into(),
                // FIXME: measure text size?
                Draw::Text(ref _x) => [1.0, 1.0].into(),
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
    pub fn render(&mut self, pass: &mut RenderPass<'_>) {
        match self.draw {
            Draw::Sprite(ref x) => {
                self.params.apply(&mut pass.sprite(x));
            }
            Draw::NineSlice(ref x) => {
                self.params.apply(&mut pass.sprite(x));
            }
            Draw::Text(ref x) => {
                // TODO: custom position
                pass.txt(self.params.pos, &x.txt);
            }
        }
    }
}

/// Common geometry data that animations can operate on
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
    /// Sets up quad parameters
    pub fn apply<'a, 'b: 'a, B: QuadParamsBuilder>(&self, builder: &'b mut B) -> &'a mut B {
        builder
            .dst_pos_px(self.pos)
            .dst_size_px(self.size)
            .color(self.color)
    }
}

/// [`Node`] surface
#[derive(Debug, Clone)]
pub enum Draw {
    Sprite(SpriteData),
    NineSlice(NineSliceSprite),
    Text(Text),
}

macro_rules! impl_into_draw {
    ($ty:ident, $var:ident) => {
        impl From<$ty> for Draw {
            fn from(x: $ty) -> Draw {
                Draw::$var(x)
            }
        }

        impl From<$ty> for Node {
            fn from(x: $ty) -> Node {
                Node::from(Draw::from(x))
            }
        }

        impl From<&$ty> for Draw {
            fn from(x: &$ty) -> Draw {
                Draw::$var(x.clone())
            }
        }

        impl From<&$ty> for Node {
            fn from(x: &$ty) -> Node {
                Node::from(Draw::from(x.clone()))
            }
        }
    };
}

impl_into_draw!(SpriteData, Sprite);
impl_into_draw!(NineSliceSprite, NineSlice);
impl_into_draw!(Text, Text);

impl Draw {
    // pub fn draw(
}

/// [`Draw`] variant
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Text {
    // TODO: unsafe tetx reference?
    pub txt: String,
    // TODO: decoration information (spans for colors, etc)
}

// /// X/Y aligment
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub struct Align2d {
//     pub h: AlignH,
//     pub v: AlignV,
// }

// impl Align2d {
//     pub fn new(h: AlignH, v: AlignV) -> Self {
//         Self { h, v }
//     }
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum AlignH {
//     Left,
//     Center,
//     Right,
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum AlignV {
//     Top,
//     Center,
//     Bottom,
// }
