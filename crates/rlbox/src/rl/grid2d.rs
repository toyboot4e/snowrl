/*!
Desrete geometry types
*/

use auto_ops::*;

pub use snow2d::input::{Dir4, Dir8, Sign};

// --------------------------------------------------------------------------------
// Space

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Rect2i {
    pos: Vec2i,
    size: Vec2u,
}

impl Rect2i {
    pub fn new(xs: impl Into<[i32; 2]>, ws: impl Into<[u32; 2]>) -> Self {
        let xs = xs.into();
        let ws = ws.into();
        Self {
            pos: Vec2i::new(xs[0], xs[1]),
            size: Vec2u::new(ws[0], ws[1]),
        }
    }

    pub fn size_u(&self) -> Vec2u {
        self.size
    }

    pub fn size_i(&self) -> Vec2i {
        Vec2i::new(self.pos.x as i32, self.pos.y as i32)
    }

    pub fn w(&self) -> u32 {
        self.size.x
    }

    pub fn h(&self) -> u32 {
        self.size.y
    }

    pub fn up(&self) -> i32 {
        self.pos.y
    }

    pub fn left(&self) -> i32 {
        self.pos.x
    }

    pub fn left_up(&self) -> Vec2i {
        self.pos
    }

    // EXCLUSIVE or not

    // pub fn right_up(&self) -> Vec2i {
    //     Vec2i::new(self.pos.x + self.size.x as i32, self.pos.y)
    // }

    // pub fn left_down(&self) -> Vec2i {
    //     Vec2i::new(self.pos.x, self.pos.y + self.size.y as i32)
    // }

    // pub fn right_down(&self) -> Vec2i {
    //     Vec2i::new(
    //         self.pos.x + self.size.x as i32,
    //         self.pos.y + self.size.y as i32,
    //     )
    // }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Vec2i {
    pub x: i32,
    pub y: i32,
}

impl_op_ex!(-|me: &Vec2i| -> Vec2i { Vec2i::new(-me.x, -me.y) });

impl_op_ex!(+ |lhs: &Vec2i, rhs: &Vec2i| -> Vec2i { Vec2i::new(lhs.x + rhs.x, lhs.y + rhs.y) });
impl_op_ex!(-|lhs: &Vec2i, rhs: &Vec2i| -> Vec2i { Vec2i::new(lhs.x - rhs.x, lhs.y - rhs.y) });
impl_op_ex!(*|lhs: &Vec2i, rhs: &Vec2i| -> Vec2i { Vec2i::new(lhs.x * rhs.x, lhs.y * rhs.y) });
impl_op_ex!(/ |lhs: &Vec2i, rhs: &Vec2i| -> Vec2i { Vec2i::new(lhs.x / rhs.x, lhs.y / rhs.y) });

impl_op_ex!(+= |lhs: &mut Vec2i, rhs: &Vec2i| { lhs.x += rhs.x; lhs.y += rhs.y; });
impl_op_ex!(-= |lhs: &mut Vec2i, rhs: &Vec2i| { lhs.x -= rhs.x; lhs.y -= rhs.y; });
impl_op_ex!(*= |lhs: &mut Vec2i, rhs: &Vec2i| { lhs.x *= rhs.x; lhs.y *= rhs.y; });
impl_op_ex!(/= |lhs: &mut Vec2i, rhs: &Vec2i| { lhs.x /= rhs.x; lhs.y /= rhs.y; });

impl_op_ex!(*|lhs: &Vec2i, rhs: &i32| -> Vec2i { Vec2i::new(lhs.x * rhs, lhs.y * rhs) });
impl_op_ex!(*|lhs: &i32, rhs: &Vec2i| -> Vec2i { Vec2i::new(rhs.x * lhs, rhs.y * lhs) });
impl_op_ex!(/|lhs: &Vec2i, rhs: &i32| -> Vec2i { Vec2i::new(lhs.x / rhs, lhs.y / rhs) });
impl_op_ex!(*= |lhs: &mut Vec2i, rhs: &i32| { lhs.x *= rhs; lhs.y *= rhs; });
impl_op_ex!(/= |lhs: &mut Vec2i, rhs: &i32| { lhs.x /= rhs; lhs.y /= rhs; });

impl Vec2i {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn from_dir4(d: Dir4) -> Self {
        Self {
            x: d.x_sign().to_i32(),
            y: d.y_sign().to_i32(),
        }
    }

    pub fn from_dir8(d: Dir8) -> Self {
        Self {
            x: d.x_sign().to_i32(),
            y: d.y_sign().to_i32(),
        }
    }

    pub fn len_rock(&self) -> u32 {
        (self.x.abs() + self.y.abs()) as u32
    }

    pub fn len_king(&self) -> u32 {
        std::cmp::max(self.x.abs(), self.y.abs()) as u32
    }

    pub fn len_f32_squared(&self) -> f32 {
        (self.x * self.x + self.y * self.y) as f32
    }

    pub fn len_f32(&self) -> f32 {
        self.len_f32_squared().sqrt()
    }
}

impl Into<[i32; 2]> for Vec2i {
    fn into(self) -> [i32; 2] {
        [self.x, self.y]
    }
}

impl Into<[i32; 2]> for &Vec2i {
    fn into(self) -> [i32; 2] {
        [self.x, self.y]
    }
}

impl From<[i32; 2]> for Vec2i {
    fn from(xs: [i32; 2]) -> Self {
        Self::new(xs[0], xs[1])
    }
}

impl From<&[i32; 2]> for Vec2i {
    fn from(xs: &[i32; 2]) -> Self {
        Self::new(xs[0], xs[1])
    }
}

impl From<Dir4> for Vec2i {
    fn from(x: Dir4) -> Self {
        Self::new(x.x_sign().to_i32(), x.y_sign().to_i32())
    }
}

impl From<Dir8> for Vec2i {
    fn from(x: Dir8) -> Self {
        Self::new(x.x_sign().to_i32(), x.y_sign().to_i32())
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Vec2u {
    pub x: u32,
    pub y: u32,
}

impl Vec2u {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

impl_op_ex!(+ |lhs: &Vec2u, rhs: &Vec2u| -> Vec2u { Vec2u::new(lhs.x + rhs.x, lhs.y + rhs.y) });
impl_op_ex!(-|lhs: &Vec2u, rhs: &Vec2u| -> Vec2u { Vec2u::new(lhs.x - rhs.x, lhs.y - rhs.y) });
impl_op_ex!(*|lhs: &Vec2u, rhs: &Vec2u| -> Vec2u { Vec2u::new(lhs.x * rhs.x, lhs.y * rhs.y) });
impl_op_ex!(/ |lhs: &Vec2u, rhs: &Vec2u| -> Vec2u { Vec2u::new(lhs.x / rhs.x, lhs.y / rhs.y) });

impl_op_ex!(+= |lhs: &mut Vec2u, rhs: &Vec2u| { lhs.x += rhs.x; lhs.y += rhs.y; });
impl_op_ex!(-= |lhs: &mut Vec2u, rhs: &Vec2u| { lhs.x -= rhs.x; lhs.y -= rhs.y; });
impl_op_ex!(*= |lhs: &mut Vec2u, rhs: &Vec2u| { lhs.x *= rhs.x; lhs.y *= rhs.y; });
impl_op_ex!(/= |lhs: &mut Vec2u, rhs: &Vec2u| { lhs.x /= rhs.x; lhs.y /= rhs.y; });

impl_op_ex!(*|lhs: &Vec2u, rhs: &u32| -> Vec2u { Vec2u::new(lhs.x * rhs, lhs.y * rhs) });
impl_op_ex!(*|lhs: &u32, rhs: &Vec2u| -> Vec2u { Vec2u::new(rhs.x * lhs, rhs.y * lhs) });
impl_op_ex!(/|lhs: &Vec2u, rhs: &u32| -> Vec2u { Vec2u::new(lhs.x / rhs, lhs.y / rhs) });
impl_op_ex!(*= |lhs: &mut Vec2u, rhs: &u32| { lhs.x *= rhs; lhs.y *= rhs; });
impl_op_ex!(/= |lhs: &mut Vec2u, rhs: &u32| { lhs.x /= rhs; lhs.y /= rhs; });

impl Into<[u32; 2]> for Vec2u {
    fn into(self) -> [u32; 2] {
        [self.x, self.y]
    }
}

impl Into<[u32; 2]> for &Vec2u {
    fn into(self) -> [u32; 2] {
        [self.x, self.y]
    }
}

impl From<[u32; 2]> for Vec2u {
    fn from(xs: [u32; 2]) -> Self {
        Self::new(xs[0], xs[1])
    }
}

impl From<&[u32; 2]> for Vec2u {
    fn from(xs: &[u32; 2]) -> Self {
        Self::new(xs[0], xs[1])
    }
}
