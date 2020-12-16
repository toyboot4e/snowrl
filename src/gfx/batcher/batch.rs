//! Batcher

use rokol::gfx::{self as rg, BakedResource};

use crate::gfx::{
    batcher::{draw::QuadParamsBuilder, mesh::DynamicMesh, QuadData, N_QUADS},
    texture::TextureData2d,
};

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
        // FIXME:
        unsafe {
            self.mesh
                .upload_vert_slice(self.buffer_offset, self.quad_ix);
        }

        self.mesh.draw(0, 6 * self.quad_ix as u32);

        self.quad_ix = 0;
        // self.mesh.
    }

    pub fn push_sprite(img: rg::Image, quad: impl Into<QuadData>) {
        // self.batch.mesh_mut().bind_image(self.tex_1.img, 0);
        // self.batch.push_quad([
        //     // pos, color, uv
        //     ([200.0, 200.0], white, [0.0, 0.0]).into(),
        //     ([400.0, 200.0], white, [1.0, 0.0]).into(),
        //     ([200.0, 400.0], white, [0.0, 1.0]).into(),
        //     ([400.0, 400.0], white, [1.0, 1.0]).into(),
        // ]);
    }

    pub fn mesh_mut(&mut self) -> &mut DynamicMesh<QuadData> {
        &mut self.mesh
    }

    pub fn push_quad(&mut self, quad: impl Into<QuadData>) {
        self.mesh.verts[self.quad_ix] = quad.into();
        self.quad_ix += 1;
    }

    pub fn next_quad_mut(&mut self) -> &mut QuadData {
        let ix = self.quad_ix;
        self.quad_ix += 1;
        &mut self.mesh.verts[ix]
    }
}

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
