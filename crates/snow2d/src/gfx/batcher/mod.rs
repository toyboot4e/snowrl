//! Batcher

pub mod draw;
pub mod mesh;
pub mod vertex;

use rokol::gfx as rg;

use crate::gfx::batcher::{
    draw::*,
    mesh::DynamicMesh,
    vertex::{QuadData, N_QUADS},
};

/// Creates index buffer for quadliterals
///
/// ```no_run
/// 0-1
/// |/|
/// 2-3
/// ```
///
/// TODO: cull mode (indices should be in reverse?)
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

/// Internal batch utility
#[derive(Debug, Clone)]
pub struct Batch {
    /// Each item of `mesh.verts` is actually [`QuadData`]
    pub mesh: DynamicMesh<QuadData>,
    /// Index of next quad
    quad_ix: usize,
    buffer_offset: i32,
    img: Option<rg::Image>,
    pub params: QuadParams,
}

impl Batch {
    pub fn new() -> Self {
        let mesh = DynamicMesh::new_16(
            vec![QuadData::default(); N_QUADS],
            &gen_quad_indices!(u16, N_QUADS)[0..],
        );

        Self {
            mesh,
            quad_ix: 0,
            buffer_offset: 0,
            img: None,
            params: Default::default(),
        }
    }

    pub fn flush(&mut self) {
        if self.quad_ix == 0 {
            return;
        }

        if self.img.is_none() {
            log::error!("no image on flushing batch");
            return;
        }

        self.draw();

        self.quad_ix = 0;
        self.img = None;
        self.buffer_offset = 0;

        // NOTE:
        self.mesh.bind.vertex_buffer_offsets[0] = 0;
    }

    fn draw(&mut self) {
        // TODO: can we not upload more than N_QUADS a frame?
        // println!(
        //     "draw: ix = {}, offset = {}, GPU offset = {}",
        //     self.quad_ix, self.buffer_offset, self.mesh.bind.vertex_buffer_offsets[0]
        // );

        {
            let offset = self
                .mesh
                .append_vert_slice(self.buffer_offset, self.quad_ix);
            self.buffer_offset = offset;
        }

        self.mesh.bind_image(self.img.unwrap(), 0);
        self.mesh.draw(0, 6 * self.quad_ix as u32);
    }

    pub fn next_quad_ix(&mut self, img: rg::Image) -> usize {
        // flush if needed
        if let Some(prev) = self.img {
            // FIXME: this guard is not working somehow (N_QUADS = 2048)
            if prev.id != img.id || (self.quad_ix + 1) >= N_QUADS {
                self.flush();
            }
        }

        self.img = Some(img);

        let ix = self.quad_ix;
        self.quad_ix += 1;
        ix
    }

    pub fn force_set_img(&mut self, img: rg::Image) {
        self.img = Some(img);
    }
}
