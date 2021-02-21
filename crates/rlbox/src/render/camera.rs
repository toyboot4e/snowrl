/*!
2D camera
*/

use glam::{Mat4, Vec3};
use snow2d::gfx::geom2d::*;

/// TODO: use it? Transfrom of position, rotation and scale
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

    pub fn to_mat4(&self) -> Mat4 {
        Mat4::from_translation(Vec3::new(-self.params.pos.x, -self.params.pos.y, 0.0))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FollowCamera2d {
    /// Deadzoine in screen coordinates (fixed position)
    pub deadzone: Rect2f,
    /// Every frame we add `target_delta * lerp` to the camera position
    pub lerp: f32,
    // pub ease: ez::Ease,
}

pub fn update_follow(cam: &mut Camera2d, follow: &FollowCamera2d, player_pos_world: Vec2f) {
    let deadzone_world = follow.deadzone.offset(cam.params.pos);

    let dx1 = player_pos_world.x - deadzone_world.right();
    let dx2 = player_pos_world.x - deadzone_world.left();

    let dx = if dx1 > 0.0 {
        dx1
    } else if dx2 < 0.0 {
        dx2
    } else {
        0.0
    };

    let dy1 = player_pos_world.y - deadzone_world.down();
    let dy2 = player_pos_world.y - deadzone_world.up();

    let dy = if dy1 > 0.0 {
        dy1
    } else if dy2 < 0.0 {
        dy2
    } else {
        0.0
    };

    log::trace!("{:?}, {:?}", dx, dy);

    cam.params.pos += Vec2f::new(dx * follow.lerp, dy * follow.lerp);
}
