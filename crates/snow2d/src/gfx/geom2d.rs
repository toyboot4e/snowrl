// 2D geometry types

use auto_ops::*;

bitflags::bitflags! {
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

// TODO: refactor saceld/unscaled API
#[derive(Debug, Clone)]
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

/// Skew matrix
///
/// Top-left and bottom-right.
#[derive(Debug, Clone, PartialEq, Default, Copy)]
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
#[derive(Debug, Clone, PartialEq, Default, Copy)]
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
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec2f {
    pub x: f32,
    pub y: f32,
}

impl Vec2f {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub fn one() -> Self {
        Self { x: 1.0, y: 1.0 }
    }

    pub fn round(&mut self) {
        self.x = self.x.round();
        self.y = self.y.round();
    }

    pub fn scale(&self, scale: impl Into<Vec2f>) -> Self {
        let scale = scale.into();

        Self {
            x: self.x * scale.x,
            y: self.y * scale.y,
        }
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

// Vec2f, f32
impl_op_ex!(-|me: &Vec2f| -> Vec2f { Vec2f::new(-me.x, -me.y) });

// Vec2f, f32
impl_op_ex!(*|lhs: &Vec2f, rhs: &f32| -> Vec2f { Vec2f::new(lhs.x * rhs, lhs.y * rhs) });
impl_op_ex!(/|lhs: &Vec2f, rhs: &f32| -> Vec2f { Vec2f::new(lhs.x / rhs, lhs.y / rhs) });
impl_op_ex!(*= |lhs: &mut Vec2f, rhs: &f32| { lhs.x *= rhs; lhs.y *= rhs; });
impl_op_ex!(/= |lhs: &mut Vec2f, rhs: &f32| { lhs.x /= rhs; lhs.y /= rhs; });

// Vec2f, Vec2f
impl_op_ex!(+ |lhs: &Vec2f, rhs: &Vec2f| -> Vec2f { Vec2f::new(lhs.x + rhs.x, lhs.y + rhs.y) });
impl_op_ex!(-|lhs: &Vec2f, rhs: &Vec2f| -> Vec2f { Vec2f::new(lhs.x - rhs.x, lhs.y - rhs.y) });
impl_op_ex!(*|lhs: &Vec2f, rhs: &Vec2f| -> Vec2f { Vec2f::new(lhs.x * rhs.x, lhs.y * rhs.y) });
impl_op_ex!(/ |lhs: &Vec2f, rhs: &Vec2f| -> Vec2f { Vec2f::new(lhs.x / rhs.x, lhs.y / rhs.y) });

// assginments
impl_op_ex!(+= |lhs: &mut Vec2f, rhs: &Vec2f| { lhs.x += rhs.x; lhs.y += rhs.y; });
impl_op_ex!(-= |lhs: &mut Vec2f, rhs: &Vec2f| { lhs.x -= rhs.x; lhs.y -= rhs.y; });
// impl_op_ex!(*= |lhs: &mut Vec2f, rhs: &Vec2f| { lhs.x *= rhs.x; lhs.y *= rhs.y; });
// impl_op_ex!(/= |lhs: &mut Vec2f, rhs: &Vec2f| { lhs.x /= rhs.x; lhs.y /= rhs.y; });

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
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Rect2f {
    pub x: f32,
    pub y: f32,
    pub w: f32,
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

    /// Coordinate of visually up edge
    pub fn top(&self) -> f32 {
        self.y
    }

    /// Coordinate of visually down edge
    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }

    pub fn set_left(&mut self, x: f32) {
        self.x = x;
    }

    pub fn set_right(&mut self, x: f32) {
        self.x = x - self.w;
    }

    pub fn set_up(&mut self, y: f32) {
        self.y = y;
    }

    pub fn set_down(&mut self, y: f32) {
        self.y = y - self.h;
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
    pub fn center(&self) -> Vec2f {
        (self.left_up() + self.right_down()) / 2.0
    }

    /// Origin in pixels from origin in normalized coordinates
    pub fn origin_px(&self, origin: impl Into<Vec2f>) -> Vec2f {
        self.left_up() + self.size().scale(origin.into())
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

    // mutations

    /// Adds offset to `self`
    pub fn translate(&mut self, offset: impl Into<Vec2f>) {
        let v = offset.into();
        self.x += v.x;
        self.y += v.y;
    }

    /// Adjusts x value assuming `w < max - min` (be warned that this is stupid)
    pub fn clamp_x(&mut self, min: f32, max: f32) {
        if self.left() < min {
            self.set_left(min);
        }
        if self.right() > max {
            self.set_right(max)
        }
    }

    /// Adjusts y value assuming `h < max - min` (be warned that this is stupid)
    pub fn clamp_y(&mut self, min: f32, max: f32) {
        if self.top() < min {
            self.set_up(min);
        }
        if self.bottom() > max {
            self.set_down(max)
        }
    }
}

/// Interaction
impl Rect2f {
    pub fn contains(&self, pos: impl Into<Vec2f>) -> bool {
        let pos = pos.into();
        !(
            //
            pos.x < self.left()
                || self.right() < pos.x
                || pos.y < self.top()
                || self.bottom() < pos.y
        )
    }

    pub fn intersects(&self, other: &Rect2f) -> bool {
        !(self.right() < other.left()
            || other.right() < self.left()
            || self.bottom() < other.top()
            || other.bottom() < self.top())
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

/// Two-dimensional matrix that stores translation, scale and rotation information
///
/// For efficiency, it is a 3x2 matrix.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Mat2f {
    // mIJ (I: row, J: column)
    m11: f32, // x scale
    m12: f32,
    m21: f32,
    m22: f32, // y scale
    m31: f32, // x translation
    m32: f32, // y translation
}

impl Mat2f {
    pub fn new(m11: f32, m12: f32, m21: f32, m22: f32, m31: f32, m32: f32) -> Self {
        Self {
            m11,
            m12,
            m21,
            m22,
            m31,
            m32,
        }
    }

    pub fn identity() -> Self {
        Self::new(1.0, 0.0, 0.0, 1., 0.0, 0.0)
    }

    pub fn translation(&self) -> Vec2f {
        Vec2f::new(self.m31, self.m32)
    }

    pub fn set_translation(&mut self, v: impl Into<Vec2f>) {
        let v = v.into();
        self.m31 = v.x;
        self.m32 = v.y;
    }

    /// Rotation in radians
    pub fn rot(&self) -> f32 {
        self.m21.atan2(self.m11)
    }

    /// Sets rotation in radians
    pub fn set_rot(&mut self, rot: f32) {
        let cos = rot.cos();
        let sin = rot.sin();

        self.m11 = cos;
        self.m12 = sin;
        self.m21 = -sin;
        self.m22 = cos;
    }

    pub fn scale(&self) -> Vec2f {
        Vec2f::new(self.m11, self.m22)
    }

    pub fn set_scale(&mut self, scale: impl Into<Vec2f>) {
        let scale = scale.into();
        self.m11 = scale.x;
        self.m22 = scale.y;
    }
}

// Mat2f, Mat2f

impl_op_ex!(+ |lhs: &Mat2f, rhs: &Mat2f| -> Mat2f {
    Mat2f::new(
        lhs.m11 + rhs.m11,
        lhs.m12 + rhs.m12,
        lhs.m21 + rhs.m21,
        lhs.m22 + rhs.m22,
        lhs.m31 + rhs.m31,
        lhs.m32 + rhs.m32,
    )
});

impl_op_ex!(-|lhs: &Mat2f, rhs: &Mat2f| -> Mat2f {
    Mat2f::new(
        lhs.m11 - rhs.m11,
        lhs.m12 - rhs.m12,
        lhs.m21 - rhs.m21,
        lhs.m22 - rhs.m22,
        lhs.m31 - rhs.m31,
        lhs.m32 - rhs.m32,
    )
});

impl_op_ex!(*|lhs: &Mat2f, rhs: &Mat2f| -> Mat2f {
    Mat2f::new(
        (lhs.m11 * rhs.m11) + (lhs.m12 * rhs.m21),
        (lhs.m11 * rhs.m12) + (lhs.m12 * rhs.m22),
        (lhs.m21 * rhs.m11) + (lhs.m22 * rhs.m21),
        (lhs.m21 * rhs.m12) + (lhs.m22 * rhs.m22),
        (lhs.m31 * rhs.m11) + (lhs.m32 * rhs.m21) + lhs.m31,
        (lhs.m31 * rhs.m12) + (lhs.m32 * rhs.m22) + lhs.m32,
    )
});

// Mat2f, f32

impl_op_ex!(*|lhs: &Mat2f, rhs: &f32| -> Mat2f {
    Mat2f::new(
        lhs.m11 * rhs,
        lhs.m12 * rhs,
        lhs.m21 * rhs,
        lhs.m22 * rhs,
        lhs.m31 * rhs,
        lhs.m32 * rhs,
    )
});

impl_op_ex!(/|lhs: &Mat2f, rhs: &f32| -> Mat2f {
    Mat2f::new(
        lhs.m11 / rhs,
        lhs.m12 / rhs,
        lhs.m21 / rhs,
        lhs.m22 / rhs,
        lhs.m31 / rhs,
        lhs.m32 / rhs,
    )
});

// assignments

impl_op_ex!(+= |lhs: &mut Mat2f, rhs: &Mat2f| {
    lhs.m11 += rhs.m11;
    lhs.m12 += rhs.m12;
    lhs.m21 += rhs.m21;
    lhs.m22 += rhs.m22;
    lhs.m31 += rhs.m31;
    lhs.m32 += rhs.m32;
});

impl_op_ex!(-=|lhs: &mut Mat2f, rhs: &Mat2f| {
    lhs.m11 -= rhs.m11;
    lhs.m12 -= rhs.m12;
    lhs.m21 -= rhs.m21;
    lhs.m22 -= rhs.m22;
    lhs.m31 -= rhs.m31;
    lhs.m32 -= rhs.m32;
});

// FIXME:
// impl_op_ex!(*=|lhs: &mut Mat2f, rhs: &Mat2f| {
//     lhs = lhs * rhs;
// });

// TODO: should I implement `/` operator?

impl Mat2f {
    /// Creates a new rotation martrix around Z axis
    pub fn with_rot(rad: f32) -> Self {
        let mut m = Self::identity();
        m.set_rot(rad);
        m
    }

    /// Creates a new scaling matrix
    pub fn with_scale(scale: impl Into<Vec2f>) -> Self {
        let scale = scale.into();
        Self::new(scale.x, 0.0, 0.0, scale.y, 0.0, 0.0)
    }

    /// Creates a new translation matrix
    pub fn with_translation(pos: impl Into<Vec2f>) -> Self {
        let pos = pos.into();
        Self::new(1.0, 0.0, 0.0, 1.0, pos.x, pos.y)
    }
}

/// Operations
impl Mat2f {
    pub fn det(&self) -> f32 {
        self.m11 * self.m22 - self.m12 * self.m21
    }

    pub fn inv(&self) -> Self {
        let det = 1.0 / self.det();

        let mut m = Self::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        m.m11 = self.m22 * det;
        m.m12 = -self.m12 * det;
        m.m21 = -self.m21 * det;
        m.m22 = self.m11 * det;
        m.m31 = (self.m32 * self.m21 - self.m31 * self.m22) * det;
        m.m32 = -(self.m32 * self.m11 - self.m31 * self.m12) * det;
        m
    }

    /// Creates a new matrix that contains linear interpolation of the values in specified matrixes
    pub fn lerp(&mut self, other: &Mat2f, amount: f32) {
        self.m11 = self.m11 + ((other.m11 - self.m11) * amount);
        self.m12 = self.m12 + ((other.m12 - self.m12) * amount);

        self.m21 = self.m21 + ((other.m21 - self.m21) * amount);
        self.m22 = self.m22 + ((other.m22 - self.m22) * amount);

        self.m31 = self.m31 + ((other.m31 - self.m31) * amount);
        self.m32 = self.m32 + ((other.m32 - self.m32) * amount);
    }

    pub fn transpose(&mut self) {
        // swap non-diagnoal elements
        let tmp = self.m12;
        self.m12 = self.m21;
        self.m21 = tmp;

        self.m31 = 0.0;
        self.m32 = 0.0;
    }
}
