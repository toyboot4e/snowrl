//! Vertex data type for `snow2d`

pub const N_QUADS: usize = 2048;

/// `snow2d` vertex data type
#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct VertexData {
    pub pos: [f32; 2],
    pub color: [u8; 4],
    pub uv: [f32; 2],
}

impl<Pos, Color, Uv> From<(Pos, Color, Uv)> for VertexData
where
    Pos: Into<[f32; 2]>,
    Color: Into<[u8; 4]>,
    Uv: Into<[f32; 2]>,
{
    fn from(data: (Pos, Color, Uv)) -> Self {
        Self {
            pos: data.0.into(),
            color: data.1.into(),
            uv: data.2.into(),
        }
    }
}

/// `snow2d` quad data type
#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct QuadData(pub [VertexData; 4]);

impl std::ops::Index<usize> for QuadData {
    type Output = VertexData;

    fn index(&self, ix: usize) -> &Self::Output {
        &self.0[ix]
    }
}

impl std::ops::IndexMut<usize> for QuadData {
    fn index_mut(&mut self, ix: usize) -> &mut Self::Output {
        &mut self.0[ix]
    }
}

impl From<[VertexData; 4]> for QuadData {
    fn from(data: [VertexData; 4]) -> Self {
        Self(data)
    }
}

impl From<&[VertexData; 4]> for QuadData {
    fn from(data: &[VertexData; 4]) -> Self {
        Self(data.clone())
    }
}
