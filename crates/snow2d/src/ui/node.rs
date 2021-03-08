/*!
UI nodes (renderables)

[`Handle`] of UI nodes are strong references, so a node won't freed until nothing refers to it.
*/

use crate::{
    gfx::{draw::*, geom2d::*, Color, RenderPass},
    utils::pool::{Handle, WeakHandle},
};

// Re-exported as [`Node`] variants
pub use crate::gfx::tex::{NineSliceSprite, SpriteData};

/// Rendering order [0, 1] (the higher, the latter)
pub type Order = f32;

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
    pub fn setup_quad<'a, 'b: 'a, B: QuadParamsBuilder>(&self, builder: &'b mut B) -> &'a mut B {
        builder
            .dst_pos_px(self.pos)
            .dst_size_px(self.size)
            .color(self.color)
    }

    pub fn transform_mut(&self, other: &mut DrawParams) {
        other.pos += self.pos;
    }
}

/// [`Node`] surface
#[derive(Debug, Clone)]
pub enum Draw {
    Sprite(SpriteData),
    NineSlice(NineSliceSprite),
    Text(Text),
    /// The node is only for parenting
    None,
}

/// DrawVariant -> Draw -> Node
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

/// [`Draw`] variant
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Text {
    // TODO: unsafe tetx reference?
    pub txt: String,
    // TODO: decoration information (spans for colors, etc)
}

/// Visible object in a UI layer
#[derive(Debug, Clone)]
pub struct Node {
    pub draw: Draw,
    /// Common geometry data
    pub params: DrawParams,
    /// Draw parameter calculated befre rendering
    pub(super) cache: DrawParams,
    /// Rendering order [0, 1] (the higher, the latter)
    pub order: Order,
    /// NOTE: Parents are alive if any children is alive
    pub(super) parent: Option<Handle<Node>>,
    pub(super) children: Vec<WeakHandle<Node>>,
    // TODO: dirty flag,
}

impl From<Draw> for Node {
    fn from(draw: Draw) -> Self {
        let params = DrawParams {
            size: match draw {
                // FIXME: parent box size. Node builder?
                Draw::None => [1.0, 1.0].into(),
                Draw::Sprite(ref x) => x.sub_tex_size_scaled().into(),
                Draw::NineSlice(ref x) => x.sub_tex_size_scaled().into(),
                // FIXME: measure text size?
                Draw::Text(ref _x) => [1.0, 1.0].into(),
            },
            ..Default::default()
        };
        Node {
            draw,
            params: params.clone(),
            cache: params.clone(),
            order: 1.0,
            children: vec![],
            parent: None,
        }
    }
}

impl Node {
    pub fn render(&mut self, pass: &mut RenderPass<'_>) {
        let params = &self.cache;
        match self.draw {
            Draw::Sprite(ref x) => {
                params.setup_quad(&mut pass.sprite(x));
            }
            Draw::NineSlice(ref x) => {
                params.setup_quad(&mut pass.sprite(x));
            }
            Draw::Text(ref x) => {
                // TODO: custom position
                pass.text(params.pos, &x.txt);
            }
            Draw::None => {}
        }
    }
}

// pub struct NodeBuilder {
//     draw: Draw,
//     params: DrawParams,
//     children:Vec<Node>,
// }

// impl NodeBuilder {
// }
