/*!
2D camera
*/

use glam::{Mat4, Vec3};
use snow2d::{gfx::geom2d::*, utils::Inspect};

/// TODO: use it? Transfrom of position, rotation and scale
#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct Transform2d {
    local: Mat2f,
    world: Mat2f,
    local_dirty: bool,
    world_dirty: bool,
}

#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct Transform2dParams {
    pub pos: Vec2f,
    pub scale: Vec2f,
    pub rot: f32,
}

impl Transform2dParams {
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

#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct Camera2d {
    pub params: Transform2dParams,
    pub size: Vec2f,
}

impl Camera2d {
    /// Bounding rectangle in pixels
    pub fn bounds(&self) -> Rect2f {
        Rect2f {
            x: self.params.pos.x.floor(),
            y: self.params.pos.y.floor(),
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
        Mat4::from_translation(Vec3::new(
            -self.params.pos.x.floor(),
            -self.params.pos.y.floor(),
            0.0,
        ))
    }
}

/// All rectangles are in pixels
#[derive(Debug, Clone, PartialEq, Inspect)]
pub struct FollowCamera2d {
    /// When the player is out of this rectangle, the camera starts to scroll
    pub sense_pads: Vec2f,
    /// The player will be at the edge of this rectangle after scrolling
    pub target_pads: Vec2f,
    /// Do not scroll out of this rectangle
    pub deadzone: Rect2f,
    pub lerp_speed: f32,
    pub is_moving: bool,
}

#[inline]
fn pads_to_rect_in_world(pads: Vec2f, viewport_size: Vec2f, offset: Vec2f) -> Rect2f {
    Rect2f::new(
        offset.x + pads.x,
        offset.y + pads.y,
        viewport_size.x - pads.x * 2.0,
        viewport_size.y - pads.y * 2.0,
    )
}

impl FollowCamera2d {
    /// * `player`: player position in world coordinates
    pub fn update_follow(&mut self, cam: &mut Camera2d, player: Vec2f, viewport_size: Vec2f) {
        // don't start scrolling if the player is inside the sense rectangle
        if !self.is_moving {
            let sense = pads_to_rect_in_world(self.sense_pads, viewport_size, cam.params.pos);
            if sense.contains(player) {
                return;
            }
        }

        let target = pads_to_rect_in_world(self.target_pads, viewport_size, cam.params.pos);

        // delta scroll to put the player in the target rectangle
        let dpos = {
            let dx1 = player.x - target.right();
            let dx2 = player.x - target.left();

            let dx = if dx1 > 0.0 {
                dx1
            } else if dx2 < 0.0 {
                dx2
            } else {
                0.0
            };

            let dy1 = player.y - target.down();
            let dy2 = player.y - target.up();

            let dy = if dy1 > 0.0 {
                dy1
            } else if dy2 < 0.0 {
                dy2
            } else {
                0.0
            };

            Vec2f::new(dx, dy)
        };

        let cam_rect = Rect2f::from([cam.params.pos, cam.size]);

        let mut target_cam_rect = cam_rect.clone();
        target_cam_rect.offset_mut(dpos);
        target_cam_rect.clamp_x_mut(self.deadzone.left(), self.deadzone.right());
        target_cam_rect.clamp_y_mut(self.deadzone.up(), self.deadzone.down());

        let dpos_clamped = target_cam_rect.left_up() - cam_rect.left_up();

        if dpos_clamped.x.abs() >= 1.0 || dpos_clamped.y.abs() >= 1.0 {
            self.is_moving = true;
            // exponential tween
            cam.params.pos += self.lerp_speed * dpos_clamped;
        } else {
            self.is_moving = false;
            // avoid sub pixel issues
            cam.params.pos += dpos_clamped;
            // cam.params.pos.floor_mut();
            cam.params.pos.round_mut();
        }
    }
}
