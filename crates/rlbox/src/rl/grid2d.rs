//! Desrete geometry types

use auto_ops::*;

// --------------------------------------------------------------------------------
// Axis

// TODO: put axis types in `rokol::input`

/// Pos | Neg | Neutral
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Sign {
    /// Positive
    Pos,
    /// Negative
    Neg,
    /// Neutral
    Neutral,
}

impl Sign {
    pub fn to_i8(&self) -> i32 {
        match self {
            Sign::Pos => 1,
            Sign::Neg => -1,
            Sign::Neutral => 0,
        }
    }

    pub fn to_i32(&self) -> i32 {
        match self {
            Sign::Pos => 1,
            Sign::Neg => -1,
            Sign::Neutral => 0,
        }
    }

    pub fn to_i64(&self) -> i64 {
        match self {
            Sign::Pos => 1,
            Sign::Neg => -1,
            Sign::Neutral => 0,
        }
    }

    pub fn to_isize(&self) -> isize {
        match self {
            Sign::Pos => 1,
            Sign::Neg => -1,
            Sign::Neutral => 0,
        }
    }

    pub fn inv(&self) -> Self {
        match self {
            Sign::Pos => Sign::Neg,
            Sign::Neg => Sign::Pos,
            Sign::Neutral => Sign::Neutral,
        }
    }
}

/// One of the four directions: N, E, S, W
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dir4 {
    N,
    E,
    S,
    W,
}

impl Dir4 {
    pub fn x_sign(&self) -> Sign {
        use Dir4::*;
        use Sign::*;

        match self {
            E => Pos,
            N | S => Neutral,
            W => Neg,
        }
    }

    pub fn y_sign(&self) -> Sign {
        use Dir4::*;
        use Sign::*;

        match self {
            N => Pos,
            E | W => Neutral,
            S => Neg,
        }
    }

    pub fn signs(&self) -> [Sign; 2] {
        [self.x_sign(), self.y_sign()]
    }

    pub fn signs_i32(&self) -> [i32; 2] {
        [self.x_sign().to_i32(), self.y_sign().to_i32()]
    }

    pub fn signs_i64(&self) -> [i64; 2] {
        [self.x_sign().to_i64(), self.y_sign().to_i64()]
    }

    pub fn signs_isize(&self) -> [isize; 2] {
        [self.x_sign().to_isize(), self.y_sign().to_isize()]
    }
}

impl Dir4 {
    pub fn inv(&self) -> Dir4 {
        match self {
            Dir4::N => Dir4::S,
            Dir4::E => Dir4::W,
            Dir4::S => Dir4::N,
            Dir4::W => Dir4::E,
        }
    }
}

/// One of the eight directions: N, NE, E, SE, ..
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dir8 {
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
    NW,
}

impl Dir8 {
    pub fn from_signs(signs: [Sign; 2]) -> Option<Self> {
        let x = signs[0].to_i8();
        let y = signs[1].to_i8();

        Some(match [x, y] {
            [0, 0] => return None,
            // clockwise
            [0, -1] => Dir8::N,
            [1, -1] => Dir8::NE,
            [1, 0] => Dir8::E,
            [1, 1] => Dir8::SE,
            [0, 1] => Dir8::S,
            [-1, 1] => Dir8::SW,
            [-1, 0] => Dir8::W,
            [-1, -1] => Dir8::NW,
            _ => unreachable!(),
        })
    }

    pub fn x_sign(&self) -> Sign {
        use Dir8::*;
        use Sign::*;

        match self {
            W | NW | SW => Neg,
            E | NE | SE => Pos,
            N | S => Neutral,
        }
    }

    pub fn y_sign(&self) -> Sign {
        use Dir8::*;
        use Sign::*;

        match self {
            N | NE | NW => Neg,
            S | SE | SW => Pos,
            E | W => Neutral,
        }
    }

    pub fn signs(&self) -> [Sign; 2] {
        [self.x_sign(), self.y_sign()]
    }

    pub fn signs_i32(&self) -> [i32; 2] {
        [self.x_sign().to_i32(), self.y_sign().to_i32()]
    }

    pub fn signs_i64(&self) -> [i64; 2] {
        [self.x_sign().to_i64(), self.y_sign().to_i64()]
    }

    pub fn signs_isize(&self) -> [isize; 2] {
        [self.x_sign().to_isize(), self.y_sign().to_isize()]
    }
}

impl Dir8 {
    pub const fn clockwise() -> &'static [Dir8; 8] {
        use Dir8::*;

        &[N, NE, E, SE, S, SW, W, NW]
    }

    pub fn inv(&self) -> Self {
        match self {
            Dir8::N => Dir8::S,
            Dir8::NE => Dir8::SW,
            Dir8::E => Dir8::W,
            Dir8::SE => Dir8::NW,
            Dir8::S => Dir8::N,
            Dir8::SW => Dir8::NE,
            Dir8::W => Dir8::E,
            Dir8::NW => Dir8::SE,
        }
    }

    pub fn r45(&self) -> Self {
        match self {
            Dir8::N => Dir8::NE,
            Dir8::NE => Dir8::E,
            Dir8::E => Dir8::SE,
            Dir8::SE => Dir8::S,
            Dir8::S => Dir8::SW,
            Dir8::SW => Dir8::W,
            Dir8::W => Dir8::NW,
            Dir8::NW => Dir8::N,
        }
    }

    pub fn l45(&self) -> Self {
        match self {
            Dir8::N => Dir8::NW,
            Dir8::NE => Dir8::W,
            Dir8::E => Dir8::NE,
            Dir8::SE => Dir8::E,
            Dir8::S => Dir8::SE,
            Dir8::SW => Dir8::SW,
            Dir8::W => Dir8::SW,
            Dir8::NW => Dir8::W,
        }
    }

    pub fn r90(&self) -> Self {
        match self {
            Dir8::N => Dir8::E,
            Dir8::NE => Dir8::SE,
            Dir8::E => Dir8::S,
            Dir8::SE => Dir8::SW,
            Dir8::S => Dir8::W,
            Dir8::SW => Dir8::NW,
            Dir8::W => Dir8::N,
            Dir8::NW => Dir8::NE,
        }
    }

    pub fn l90(&self) -> Self {
        match self {
            Dir8::N => Dir8::W,
            Dir8::NE => Dir8::NE,
            Dir8::E => Dir8::N,
            Dir8::SE => Dir8::NE,
            Dir8::S => Dir8::E,
            Dir8::SW => Dir8::SE,
            Dir8::W => Dir8::S,
            Dir8::NW => Dir8::SW,
        }
    }
}

// --------------------------------------------------------------------------------
// Space

/// Screen bounds in pixels
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

/// Size/point in pixels
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

/// Point in pixels
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
