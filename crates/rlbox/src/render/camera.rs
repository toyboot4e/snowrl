/*!
TODO: 2D camera
TODO: transform2d
*/

use snow2d::gfx::geom2d::*;

/// Transfrom of position, rotation and scale
#[derive(Debug, Clone, PartialEq)]
pub struct Transform2d {
    local: Mat2f,
    world: Mat2f,
    local_dirty: bool,
    world_dirty: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransformParams2d {
    pub pos: Vec2f,
    pub scale: Vec2f,
    pub rot: f32,
}

impl TransformParams2d {
    /// Returns transformation matrix
    pub fn to_mat(&self) -> Mat2f {
        let cos = self.rot.cos();
        let sin = self.rot.sin();
        Mat2f {
            m11: self.scale.x * cos,
            m12: sin,
            m13: self.scale.x * self.pos.x,
            m21: -sin,
            m22: self.scale.y * cos,
            m23: self.scale.y * self.pos.y,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Camera2d {
    pub params: TransformParams2d,
    pub size: Vec2f,
}

impl Camera2d {
    pub fn bounds(&self) -> Rect2f {
        Rect2f {
            x: self.params.pos.x,
            y: self.params.pos.y,
            w: self.size.x,
            h: self.size.y,
        }
    }

    /// World coordinates to screen coordinates
    pub fn w2s(&self, pos: Vec2f) -> Vec2f {
        Vec2f {
            x: pos.x - self.params.pos.x,
            y: pos.y - self.params.pos.y,
        }
    }
}
