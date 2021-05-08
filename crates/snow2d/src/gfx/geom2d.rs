//! 2D geometry types. TODO: remove this

use crate::utils::Inspect;
use auto_ops::*;
use serde::{Deserialize, Serialize};

bitflags::bitflags! {
    #[derive(Default, Deserialize, Serialize)]
    pub struct Flips: u8 {
        /// Render the sprite as it is
        const NONE = 0;
        /// Render the sprite reversed along the X axis
        const H = 1;
        /// Render the sprite reversed along the Y axis
        const V = 2;
        const HV = 3;
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Scaled<T> {
    Px(T),
    Normalized(T),
}

impl<T> Scaled<T> {
    pub fn inner(&self) -> &T {
        match self {
            Scaled::Px(x) => x,
            Scaled::Normalized(x) => x,
        }
    }
}

impl<T: Default> Default for Scaled<T> {
    fn default() -> Self {
        Self::Px(T::default())
    }
}

/// Skew matrix
///
/// Top-left and bottom-right.
#[derive(Debug, Clone, PartialEq, Default, Copy, Deserialize, Serialize, Inspect)]
pub struct Skew2f {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

impl Skew2f {
    pub fn reversed(&self) -> Self {
        Self {
            x1: -self.x1,
            y1: -self.y1,
            x2: -self.x2,
            y2: -self.y2,
        }
    }
}

/// Rotation matrix expanded from a radian value
///
/// Use radian to store rotation. Top-left and bottom-right.
#[derive(Debug, Clone, PartialEq, Default, Copy, Deserialize, Serialize, Inspect)]
pub(crate) struct Rot2f {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

impl Rot2f {
    pub fn from_rad(rad: f32) -> Self {
        // TODO: what is this..
        if rad >= f32::EPSILON {
            let sin = rad.sin();
            let cos = rad.cos();
            Self {
                x1: cos,
                y1: sin,
                x2: -sin,
                y2: cos,
            }
        } else {
            Self {
                x1: 1.0,
                y1: 0.0,
                x2: 0.0,
                y2: 1.0,
            }
        }
    }
}

/// 2D vector, intended for both positions and sizes
#[derive(Debug, Clone, Copy, PartialEq, Default, Deserialize, Serialize, Inspect)]
#[inspect(as = "[f32; 2]")]
pub struct Vec2f {
    pub x: f32,
    pub y: f32,
}

impl Vec2f {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub const ONE: Self = Self { x: 1.0, y: 1.0 };

    pub fn offset(&self, x: impl Into<Self>) -> Self {
        self + x.into()
    }

    pub fn scale(&self, scale: impl Into<Vec2f>) -> Self {
        let scale = scale.into();

        Self {
            x: self.x * scale.x,
            y: self.y * scale.y,
        }
    }

    pub fn round(&self) -> Self {
        Self::new(self.x.round(), self.y.round())
    }

    pub fn floor(&self) -> Self {
        Self::new(self.x.floor(), self.y.floor())
    }

    pub fn round_mut(&mut self) {
        self.x = self.x.round();
        self.y = self.y.round();
    }

    pub fn floor_mut(&mut self) {
        self.x = self.x.floor();
        self.y = self.y.floor();
    }

    /// Angle in radian
    pub fn rad(&self) -> f32 {
        self.y.atan2(self.x)
    }

    pub fn len_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    pub fn len(&self) -> f32 {
        self.len_squared().sqrt()
    }
}

// negation
impl_op_ex!(-|me: &Vec2f| -> Vec2f { Vec2f::new(-me.x, -me.y) });

// Vec2f, f32
impl_op_ex!(*|lhs: &Vec2f, rhs: &f32| -> Vec2f { Vec2f::new(lhs.x * rhs, lhs.y * rhs) });
impl_op_ex!(/|lhs: &Vec2f, rhs: &f32| -> Vec2f { Vec2f::new(lhs.x / rhs, lhs.y / rhs) });
impl_op_ex!(*= |lhs: &mut Vec2f, rhs: &f32| { lhs.x *= rhs; lhs.y *= rhs; });
impl_op_ex!(/= |lhs: &mut Vec2f, rhs: &f32| { lhs.x /= rhs; lhs.y /= rhs; });

// f32, Vec2f
impl_op_ex!(*|lhs: &f32, rhs: &Vec2f| -> Vec2f { Vec2f::new(lhs * rhs.x, lhs * rhs.y) });

// Vec2f, Vec2f
impl_op_ex!(+ |lhs: &Vec2f, rhs: &Vec2f| -> Vec2f { Vec2f::new(lhs.x + rhs.x, lhs.y + rhs.y) });
impl_op_ex!(-|lhs: &Vec2f, rhs: &Vec2f| -> Vec2f { Vec2f::new(lhs.x - rhs.x, lhs.y - rhs.y) });
impl_op_ex!(*|lhs: &Vec2f, rhs: &Vec2f| -> Vec2f { Vec2f::new(lhs.x * rhs.x, lhs.y * rhs.y) });
impl_op_ex!(/ |lhs: &Vec2f, rhs: &Vec2f| -> Vec2f { Vec2f::new(lhs.x / rhs.x, lhs.y / rhs.y) });

// assginments
impl_op_ex!(+= |lhs: &mut Vec2f, rhs: &Vec2f| { lhs.x += rhs.x; lhs.y += rhs.y; });
impl_op_ex!(-= |lhs: &mut Vec2f, rhs: &Vec2f| { lhs.x -= rhs.x; lhs.y -= rhs.y; });
impl_op_ex!(*= |lhs: &mut Vec2f, rhs: &Vec2f| { lhs.x *= rhs.x; lhs.y *= rhs.y; });
impl_op_ex!(/= |lhs: &mut Vec2f, rhs: &Vec2f| { lhs.x /= rhs.x; lhs.y /= rhs.y; });

// TODO: assignment with impl Into<Vec2f>

impl From<[f32; 2]> for Vec2f {
    fn from(xs: [f32; 2]) -> Self {
        Self { x: xs[0], y: xs[1] }
    }
}

impl From<&[f32; 2]> for Vec2f {
    fn from(xs: &[f32; 2]) -> Self {
        Self { x: xs[0], y: xs[1] }
    }
}

impl From<(f32, f32)> for Vec2f {
    fn from(xs: (f32, f32)) -> Self {
        Self { x: xs.0, y: xs.1 }
    }
}

impl From<&(f32, f32)> for Vec2f {
    fn from(xs: &(f32, f32)) -> Self {
        Self { x: xs.0, y: xs.1 }
    }
}

impl Into<[f32; 2]> for Vec2f {
    fn into(self) -> [f32; 2] {
        [self.x, self.y]
    }
}

impl Into<[f32; 2]> for &Vec2f {
    fn into(self) -> [f32; 2] {
        [self.x, self.y]
    }
}

impl Into<(f32, f32)> for Vec2f {
    fn into(self) -> (f32, f32) {
        (self.x, self.y)
    }
}

impl Into<(f32, f32)> for &Vec2f {
    fn into(self) -> (f32, f32) {
        (self.x, self.y)
    }
}

/// 2D rectangle
///
/// It doesn't consider offset, rotation and scales.
///
/// Note that the `x`, `y` fields correspond top-left point. If you consider an origin at somewhere
/// else, then those methods don't make sense.
#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize, Inspect)]
pub struct Rect2f {
    /// Left
    pub x: f32,
    /// Up
    pub y: f32,
    /// Width
    pub w: f32,
    /// Height
    pub h: f32,
}

impl Rect2f {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn unit() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            w: 1.0,
            h: 1.0,
        }
    }

    pub fn from_size(size: impl Into<Vec2f>) -> Self {
        let size = size.into();
        Self {
            x: 0.0,
            y: 0.0,
            w: size.x,
            h: size.y,
        }
    }

    pub fn size(&self) -> Vec2f {
        Vec2f {
            x: self.w,
            y: self.h,
        }
    }
}

/// Primitive
impl Rect2f {
    // scalars
    pub fn left(&self) -> f32 {
        self.x
    }

    pub fn right(&self) -> f32 {
        self.x + self.w
    }

    pub fn up(&self) -> f32 {
        self.y
    }

    pub fn down(&self) -> f32 {
        self.y + self.h
    }

    pub fn set_left(&mut self, left: f32) {
        self.x = left;
    }

    pub fn set_right(&mut self, right: f32) {
        self.x = right - self.w;
    }

    pub fn set_up(&mut self, up: f32) {
        self.y = up;
    }

    pub fn set_down(&mut self, down: f32) {
        self.y = down - self.h;
    }

    // vectors
    pub fn left_up(&self) -> Vec2f {
        Vec2f {
            x: self.x,
            y: self.y,
        }
    }

    pub fn right_up(&self) -> Vec2f {
        Vec2f {
            x: self.x + self.w,
            y: self.y,
        }
    }

    pub fn left_down(&self) -> Vec2f {
        Vec2f {
            x: self.x,
            y: self.y + self.h,
        }
    }

    pub fn right_down(&self) -> Vec2f {
        Vec2f {
            x: self.x + self.w,
            y: self.y + self.h,
        }
    }

    pub fn set_left_up(&mut self, pos: impl Into<[f32; 2]>) {
        let pos = pos.into();
        self.x = pos[0];
        self.y = pos[1];
    }

    pub fn set_right_up(&mut self, pos: impl Into<[f32; 2]>) {
        let pos = pos.into();
        self.x = pos[0] - self.w;
        self.y = pos[1];
    }

    pub fn set_left_down(&mut self, pos: impl Into<[f32; 2]>) {
        let pos = pos.into();
        self.x = pos[0];
        self.y = pos[1] - self.h;
    }

    pub fn set_right_down(&mut self, pos: impl Into<[f32; 2]>) {
        let pos = pos.into();
        self.x = pos[0] - self.w;
        self.y = pos[1] - self.h;
    }
}

/// More semantic
impl Rect2f {
    // getters

    pub fn center(&self) -> Vec2f {
        (self.left_up() + self.right_down()) / 2.0
    }

    /// Origin in pixels from origin in normalized coordinates
    pub fn origin_px(&self, origin: impl Into<Vec2f>) -> Vec2f {
        self.left_up() + self.size().scale(origin.into())
    }

    // convert

    pub fn offset(&self, pos: impl Into<[f32; 2]>) -> Rect2f {
        let pos = pos.into();
        Rect2f {
            x: self.x + pos[0],
            y: self.y + pos[1],
            w: self.w,
            h: self.h,
        }
    }

    // mutations

    pub fn offset_mut(&mut self, pos: impl Into<[f32; 2]>) {
        let pos = pos.into();
        self.x += pos[0];
        self.y += pos[1];
    }

    /// Sets the position of the center
    pub fn set_center(&mut self, pos: impl Into<[f32; 2]>) {
        let pos = pos.into();
        self.x = pos[0] - self.w / 2.0;
        self.y = pos[1] - self.h / 2.0;
    }

    /// Sets the position of the origin specified with normalized coordinates
    pub fn set_origin(&mut self, pos: impl Into<[f32; 2]>, origin: impl Into<[f32; 2]>) {
        let pos = pos.into();
        let origin = origin.into();
        self.x = pos[0] - self.w * origin[0];
        self.y = pos[1] - self.h * origin[1];
    }

    /// Adds offset to `self`
    pub fn translate_mut(&mut self, offset: impl Into<Vec2f>) {
        let v = offset.into();
        self.x += v.x;
        self.y += v.y;
    }

    /// Adjusts x value assuming `w < max - min` (be warned that this is stupid)
    pub fn clamp_x_mut(&mut self, min: f32, max: f32) {
        if self.left() < min {
            self.set_left(min);
        }
        if self.right() > max {
            self.set_right(max)
        }
    }

    /// Adjusts y value assuming `h < max - min` (be warned that this is stupid)
    pub fn clamp_y_mut(&mut self, min: f32, max: f32) {
        if self.up() < min {
            self.set_up(min);
        }
        if self.down() > max {
            self.set_down(max)
        }
    }
}

/// Interaction
impl Rect2f {
    pub fn contains(&self, pos: impl Into<Vec2f>) -> bool {
        let pos = pos.into();
        !(pos.x < self.left() || self.right() < pos.x || pos.y < self.up() || self.down() < pos.y)
    }

    pub fn intersects(&self, other: &Rect2f) -> bool {
        !(self.right() < other.left()
            || other.right() < self.left()
            || self.down() < other.up()
            || other.down() < self.up())
    }
}

/// ([x, y], [w, h]) -> Rect2f
impl<T, U> From<(T, U)> for Rect2f
where
    T: Into<[f32; 2]>,
    U: Into<[f32; 2]>,
{
    fn from(xs: (T, U)) -> Self {
        let (xy, wh) = xs;
        let xy = xy.into();
        let wh = wh.into();
        Self {
            x: xy[0],
            y: xy[1],
            w: wh[0],
            h: wh[1],
        }
    }
}

/// [(x, y), (w, h)] -> Rect2f
impl<T> From<[T; 2]> for Rect2f
where
    T: Into<(f32, f32)> + Copy,
{
    fn from(xs: [T; 2]) -> Self {
        let xy = xs[0].clone().into();
        let wh = xs[1].clone().into();
        Self {
            x: xy.0,
            y: xy.1,
            w: wh.0,
            h: wh.1,
        }
    }
}

/// [x, y, w, h] -> Rect2f
impl From<[f32; 4]> for Rect2f {
    fn from(xs: [f32; 4]) -> Self {
        Self {
            x: xs[0],
            y: xs[1],
            w: xs[2],
            h: xs[3],
        }
    }
}

/// Rect2f -> [x, y, w, h]
impl Into<[f32; 4]> for Rect2f {
    fn into(self) -> [f32; 4] {
        [self.x, self.y, self.w, self.h]
    }
}

/// &Rect2f -> [x, y, w, h]
impl Into<[f32; 4]> for &Rect2f {
    fn into(self) -> [f32; 4] {
        [self.x, self.y, self.w, self.h]
    }
}

/// Column-major 2x3 matrix, which represents 3x3 matrix
///
/// ```txt
///  m[0] m[1]  ...  |  scale_x  sin       0
///  m[2] m[3]  ...  |  cos      scale_y   0
///  m[4] m[5]  ...  |  trans_x  trans_y   1
/// ```
///
/// TODO: assert on zero division (NAN)?
#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize, Inspect)]
pub struct Mat2f {
    /// Scale x
    pub m11: f32,
    /// Rotation component
    pub m12: f32,
    /// Translate x
    pub m13: f32,
    /// Rotation component
    pub m21: f32,
    /// Scale y
    pub m22: f32,
    /// Translate y
    pub m23: f32,
}

impl Mat2f {
    pub const IDENTITY: Self = Self {
        m11: 1.0,
        m12: 0.0,
        m13: 0.0,
        m21: 0.0,
        m22: 1.0,
        m23: 0.0,
    };

    /// Position component
    pub fn tr(&self) -> Vec2f {
        Vec2f::new(self.m13, self.m23)
    }

    /// Rotation in radians
    pub fn rot(&self) -> f32 {
        self.m11.atan2(self.m22)
    }

    /// Rotation in radians
    pub fn scale(&self) -> Vec2f {
        Vec2f::new(self.m11, self.m22)
    }

    pub fn from_tr(tr: impl Into<[f32; 2]>) -> Self {
        let tr = tr.into();
        Self {
            m13: tr[0],
            m23: tr[1],
            ..Default::default()
        }
    }

    pub fn from_rot(rot: f32) -> Self {
        let (cos, sin) = (rot.cos(), rot.sin());
        Self {
            m11: cos,
            m12: sin,
            m21: -sin,
            m22: cos,
            ..Default::default()
        }
    }

    pub fn from_scale(scale: impl Into<[f32; 2]>) -> Self {
        let scale = scale.into();
        Self {
            m11: scale[0],
            m22: scale[1],
            ..Default::default()
        }
    }

    pub fn det(&self) -> f32 {
        self.m11 * self.m22 - self.m12 * self.m21
    }

    pub fn inv(&self) -> Self {
        let inv_det = 1.0 / self.det();
        Self {
            m11: self.m22 * inv_det,
            m12: -self.m12 * inv_det,
            m13: -(self.m12 * self.m23 - self.m13 * self.m22) * inv_det,
            m21: self.m22 * inv_det,
            m22: self.m11 * inv_det,
            m23: -(self.m11 * self.m23 - self.m13 * self.m22) * inv_det,
        }
    }

    pub fn mul(&self, other: impl std::borrow::Borrow<Self>) -> Self {
        let other = other.borrow();
        Self {
            m11: self.m11 * other.m11 + self.m12 * other.m21,
            m12: self.m11 * other.m12 + self.m12 * other.m22,
            m13: self.m11 * other.m13 + self.m12 * other.m23 + self.m13,
            m21: self.m21 * other.m11 + self.m22 * other.m21,
            m22: self.m21 * other.m12 + self.m22 * other.m22,
            m23: self.m21 * other.m13 + self.m22 * other.m23 + self.m23,
        }
    }
}

// Mat2f, Mat2f
impl_op_ex!(+ |lhs: &Mat2f, rhs: &Mat2f| -> Mat2f {
    Mat2f {
        m11: lhs.m11 + rhs.m11,
        m12: lhs.m12 + rhs.m12,
        m13: lhs.m13 + rhs.m13,
        m21: lhs.m21 + rhs.m21,
        m22: lhs.m22 + rhs.m22,
        m23: lhs.m23 + rhs.m23,
    }
});

impl_op_ex!(-|lhs: &Mat2f, rhs: &Mat2f| -> Mat2f {
    Mat2f {
        m11: lhs.m11 - rhs.m11,
        m12: lhs.m12 - rhs.m12,
        m13: lhs.m13 - rhs.m13,
        m21: lhs.m21 - rhs.m21,
        m22: lhs.m22 - rhs.m22,
        m23: lhs.m23 - rhs.m23,
    }
});

impl_op_ex!(*|lhs: &Mat2f, rhs: &Mat2f| -> Mat2f {
    Mat2f {
        m11: lhs.m11 * rhs.m11,
        m12: lhs.m12 * rhs.m12,
        m13: lhs.m13 * rhs.m13,
        m21: lhs.m21 * rhs.m21,
        m22: lhs.m22 * rhs.m22,
        m23: lhs.m23 * rhs.m23,
    }
});

impl_op_ex!(/|lhs: &Mat2f, rhs: &Mat2f| -> Mat2f {
    Mat2f {
        m11: lhs.m11 / rhs.m11,
        m12: lhs.m12 / rhs.m12,
        m13: lhs.m13 / rhs.m13,
        m21: lhs.m21 / rhs.m21,
        m22: lhs.m22 / rhs.m22,
        m23: lhs.m23 / rhs.m23,
    }
});

impl_op_ex!(+= |lhs: &mut Mat2f, rhs: &Mat2f| {
    lhs.m11 += rhs.m11;
    lhs.m12 += rhs.m12;
    lhs.m13 += rhs.m13;
    lhs.m21 += rhs.m21;
    lhs.m22 += rhs.m22;
    lhs.m23 += rhs.m23;
});

impl_op_ex!(-= |lhs: &mut Mat2f, rhs: &Mat2f| {
    lhs.m11 -= rhs.m11;
    lhs.m12 -= rhs.m12;
    lhs.m13 -= rhs.m13;
    lhs.m21 -= rhs.m21;
    lhs.m22 -= rhs.m22;
    lhs.m23 -= rhs.m23;
});

impl_op_ex!(*= |lhs: &mut Mat2f, rhs: &Mat2f| {
    lhs.m11 *= rhs.m11;
    lhs.m12 *= rhs.m12;
    lhs.m13 *= rhs.m13;
    lhs.m21 *= rhs.m21;
    lhs.m22 *= rhs.m22;
    lhs.m23 *= rhs.m23;
});

impl_op_ex!(/= |lhs: &mut Mat2f, rhs: &Mat2f| {
    lhs.m11 /= rhs.m11;
    lhs.m12 /= rhs.m12;
    lhs.m13 /= rhs.m13;
    lhs.m21 /= rhs.m21;
    lhs.m22 /= rhs.m22;
    lhs.m23 /= rhs.m23;
});

// Mat2f, f32
impl_op_ex!(+ |lhs: &Mat2f, rhs: &f32| -> Mat2f {
    Mat2f {
        m11: lhs.m11 + rhs,
        m12: lhs.m12 + rhs,
        m13: lhs.m13 + rhs,
        m21: lhs.m21 + rhs,
        m22: lhs.m22 + rhs,
        m23: lhs.m23 + rhs,
    }
});

impl_op_ex!(-|lhs: &Mat2f, rhs: &f32| -> Mat2f {
    Mat2f {
        m11: lhs.m11 - rhs,
        m12: lhs.m12 - rhs,
        m13: lhs.m13 - rhs,
        m21: lhs.m21 - rhs,
        m22: lhs.m22 - rhs,
        m23: lhs.m23 - rhs,
    }
});

impl_op_ex!(*|lhs: &Mat2f, rhs: &f32| -> Mat2f {
    Mat2f {
        m11: lhs.m11 * rhs,
        m12: lhs.m12 * rhs,
        m13: lhs.m13 * rhs,
        m21: lhs.m21 * rhs,
        m22: lhs.m22 * rhs,
        m23: lhs.m23 * rhs,
    }
});

impl_op_ex!(/|lhs: &Mat2f, rhs: &f32| -> Mat2f {
    Mat2f {
        m11: lhs.m11 / rhs,
        m12: lhs.m12 / rhs,
        m13: lhs.m13 / rhs,
        m21: lhs.m21 / rhs,
        m22: lhs.m22 / rhs,
        m23: lhs.m23 / rhs,
    }
});

impl_op_ex!(+= |lhs: &mut Mat2f, rhs: &f32| {
    lhs.m11 += rhs;
    lhs.m12 += rhs;
    lhs.m13 += rhs;
    lhs.m21 += rhs;
    lhs.m22 += rhs;
    lhs.m23 += rhs;
});

impl_op_ex!(-= |lhs: &mut Mat2f, rhs: &f32| {
    lhs.m11 -= rhs;
    lhs.m12 -= rhs;
    lhs.m13 -= rhs;
    lhs.m21 -= rhs;
    lhs.m22 -= rhs;
    lhs.m23 -= rhs;
});

impl_op_ex!(*= |lhs: &mut Mat2f, rhs: &f32| {
    lhs.m11 *= rhs;
    lhs.m12 *= rhs;
    lhs.m13 *= rhs;
    lhs.m21 *= rhs;
    lhs.m22 *= rhs;
    lhs.m23 *= rhs;
});

impl_op_ex!(/= |lhs: &mut Mat2f, rhs: &f32| {
    lhs.m11 /= rhs;
    lhs.m12 /= rhs;
    lhs.m13 /= rhs;
    lhs.m21 /= rhs;
    lhs.m22 /= rhs;
    lhs.m23 /= rhs;
});

// Mat2f, Vec2f
impl_op_ex!(*|lhs: &Mat2f, rhs: &Vec2f| -> Vec2f {
    Vec2f::new(
        rhs.x * lhs.m11 + rhs.y * lhs.m12 + lhs.m13,
        rhs.x * lhs.m21 + rhs.y * lhs.m22 + lhs.m23,
    )
});

#[cfg(test)]
mod test {
    use super::*;
    use std::f32::consts::PI;

    /// Nearly equal
    fn mat_eq(m1: &Mat2f, m2: &Mat2f) {
        assert!(m1.m11 - m2.m11 < 1.0e-6);
        assert!(m1.m12 - m2.m12 < 1.0e-6);
        assert!(m1.m13 - m2.m13 < 1.0e-6);
        assert!(m1.m21 - m2.m21 < 1.0e-6);
        assert!(m1.m22 - m2.m22 < 1.0e-6);
        assert!(m1.m23 - m2.m23 < 1.0e-6);
    }

    #[test]
    fn mat() {
        let m1 = Mat2f {
            m11: 1.0,
            m12: 2.0,
            m13: 5.0,
            m21: 2.0,
            m22: 2.0,
            m23: 4.0,
        };

        assert_eq!(m1.det(), -2.0);

        let m2 = Mat2f {
            m11: -1.0,
            m12: 1.0,
            m13: 1.0,
            m21: 1.0,
            m22: -0.5,
            m23: -3.0,
        };

        mat_eq(&m1.inv(), &m2);
    }

    #[test]
    fn rot() {
        let m1 = Mat2f::from_rot(PI / 4.0);
        let m2 = Mat2f::from_rot(PI / 2.0);
        let m3 = Mat2f::from_rot(PI / 4.0 * 3.0);
        mat_eq(&m2.mul(m1), &m3);
    }
}
