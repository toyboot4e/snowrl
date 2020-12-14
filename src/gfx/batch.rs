//! Batcher

use rokol::gfx::{self as rg, BakedResource};

use crate::gfx::{mesh::DynamicMesh, texture::TextureData2d};

const N_QUADS: usize = 2048;

#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct VertexData {
    pos: [f32; 2],
    color: [u8; 4],
    uv: [f32; 2],
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

#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct QuadData(pub [VertexData; 4]);

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

/// Creates index buffer for quadliterals
///
/// ```no_run
/// 0-1
/// |/|
/// 2-3
/// ```
///
/// TODO: cull mode (index should be opposite?)
///
/// Each index element has 16 bits length.
macro_rules! gen_quad_indices {
    ( $ix_type:ident, $n_quads:expr ) => {{
        let mut indices = [0; 6 * $n_quads as usize];

        for q in 0..$n_quads as $ix_type {
            let (i, v) = (q * 6, q * 4);
            indices[i as usize] = v as $ix_type;
            indices[(i + 1) as usize] = v + 1 as $ix_type;
            indices[(i + 2) as usize] = v + 2 as $ix_type;
            indices[(i + 3) as usize] = v + 3 as $ix_type;
            indices[(i + 4) as usize] = v + 2 as $ix_type;
            indices[(i + 5) as usize] = v + 1 as $ix_type;
        }

        indices
    }};
}

#[derive(Debug, Clone, Default)]
pub struct Batch {
    /// Each item of `mesh.verts` is actually [`QuadData`]
    mesh: DynamicMesh<QuadData>,
    /// Index of next quad
    quad_ix: usize,
    // TODO:
    buffer_offset: usize,
    img: Option<rg::Image>,
}

impl Batch {
    pub fn init(&mut self) {
        self.mesh = DynamicMesh::new_16(
            vec![QuadData::default(); N_QUADS],
            &gen_quad_indices!(u16, N_QUADS)[0..],
        );
    }

    pub fn begin(&mut self) -> BatchApi<'_> {
        BatchApi { batch: self }
    }

    pub fn flush(&mut self) {
        // TODO: span of draw calls
        self.mesh
            .append_vert_slice(self.buffer_offset, self.quad_ix);

        self.mesh.draw(0, 6 * self.quad_ix as u32);

        self.quad_ix = 0;
        // self.mesh.
    }

    pub fn mesh_mut(&mut self) -> &mut DynamicMesh<QuadData> {
        &mut self.mesh
    }

    #[inline]
    pub fn set_quad(&mut self, quad: impl Into<QuadData>) {
        self.mesh.verts[self.quad_ix] = quad.into();
        self.quad_ix += 1;
    }

    pub fn quad_mut(&mut self) -> &mut QuadData {
        let ix = self.quad_ix;
        self.quad_ix += 1;
        &mut self.mesh.verts[ix]
    }
}

// pub struct DrawCall<'a> {
//     quads: &'a [QuadData],
// }

pub struct BatchApi<'a> {
    batch: &'a mut Batch,
}

impl<'a> Drop for BatchApi<'a> {
    fn drop(&mut self) {
        self.batch.flush();
    }
}

impl<'a> BatchApi<'a> {
    pub fn sprite(&mut self, tex: &TextureData2d, mat: glam::Mat3) -> &mut Self {
        if let Some(img) = self.batch.img {
            if img.id == tex.img.id {
                self.batch.flush();
                self.batch.img = Some(tex.img);
            }
        }

        self
    }
}
