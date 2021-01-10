//! 2D geometry types. TODO: remove this

use auto_ops::*;

bitflags::bitflags! {
    #[derive(Default)]
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

impl<T: Default> Default for Scaled<T> {
    fn default() -> Self {
        Self::Px(T::default())
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
