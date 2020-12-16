/*!

Graphics

# Coordinate system

Same as OpenGL or school math (left-handed and column-major).

*/

pub mod batcher;
pub mod camera;
pub mod geom2d;
pub mod shaders;
pub mod texture;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Default for Color {
    fn default() -> Self {
        Self::WHITE
    }
}

impl Color {
    pub fn to_array(&self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }

    pub fn to_normalized_array(&self) -> [f32; 4] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        ]
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

#[macro_export]
macro_rules! def_colors {
    ($(($name:ident, $r:expr, $g:expr, $b:expr, $a:expr)), * $(,)?) => {
        impl Color {
            $(
                pub const $name: Self = Self {
                    r: $r,
                    g: $g,
                    b: $b,
                    a: $a,
                };
            )*
        }
    };
}

def_colors!(
    // name, r, g, b, a
    (WHITE, 255, 255, 255, 255),
    (BLACK, 0, 0, 0, 0),
    (GRAY, 32, 32, 32, 32),
    (CORNFLOWER_BLUE, 100, 149, 237, 255),
);

impl From<[u8; 4]> for Color {
    fn from(xs: [u8; 4]) -> Self {
        Self::rgba(xs[0], xs[1], xs[2], xs[3])
    }
}

impl From<&[u8; 4]> for Color {
    fn from(xs: &[u8; 4]) -> Self {
        Self::rgba(xs[0], xs[1], xs[2], xs[3])
    }
}

impl From<[u8; 3]> for Color {
    fn from(xs: [u8; 3]) -> Self {
        Self::rgb(xs[0], xs[1], xs[2])
    }
}

impl From<&[u8; 3]> for Color {
    fn from(xs: &[u8; 3]) -> Self {
        Self::rgb(xs[0], xs[1], xs[2])
    }
}

// TODO: define operators
