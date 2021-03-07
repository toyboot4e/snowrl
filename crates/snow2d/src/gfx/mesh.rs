/*!
Fluent API to [`rokol::gfx::Bindings`], i.e, vertex/index buffers and image slots
*/

use {
    rokol::gfx::{self as rg, BakedResource},
    std::marker::PhantomData,
};

/// Wrapper of immutable buffers
#[derive(Debug, Clone, Default)]
pub struct StaticMesh<V> {
    bind: rg::Bindings,
    n_indices: usize,
    _phantom: PhantomData<fn() -> V>,
}

impl<V> Drop for StaticMesh<V> {
    fn drop(&mut self) {
        rg::Buffer::destroy(self.bind.vertex_buffers[0]);
        rg::Buffer::destroy(self.bind.index_buffer);
    }
}

impl<V> StaticMesh<V> {
    fn new<T>(verts: &[V], indices: &[T]) -> Self {
        Self {
            bind: rg::Bindings {
                index_buffer: rg::Buffer::create(&rg::ibuf_desc_immutable(indices, "")),
                vertex_buffers: {
                    let mut xs = [Default::default(); 8];
                    xs[0] = rg::Buffer::create(&rg::vbuf_desc_immutable(verts, ""));
                    xs
                },
                ..Default::default()
            },
            n_indices: indices.len(),
            _phantom: PhantomData,
        }
    }

    /// New mesh with `u16` indices
    pub fn new_16(verts: &[V], indices: &[u16]) -> Self {
        Self::new(verts, indices)
    }

    /// New mesh with `u32` indices
    pub fn new_32(verts: &[V], indices: &[u32]) -> Self {
        Self::new(verts, indices)
    }

    /// slot: [0, 12)
    pub fn bind_image(&mut self, img: rg::Image, slot: usize) {
        self.bind.fs_images[slot] = img;
    }

    pub fn draw_all(&self) {
        rg::apply_bindings(&self.bind);
        rg::draw(0, self.n_indices as u32, 1);
    }
}

/// Wrapper of dynamic buffers
#[derive(Debug, Clone, Default)]
pub struct DynamicMesh<V> {
    pub bind: rg::Bindings,
    pub n_indices: usize,
    pub verts: Vec<V>,
    _phantom: PhantomData<fn() -> V>,
}

impl<V> Drop for DynamicMesh<V> {
    fn drop(&mut self) {
        rg::Buffer::destroy(self.bind.vertex_buffers[0]);
        rg::Buffer::destroy(self.bind.index_buffer);
    }
}

impl<V> DynamicMesh<V> {
    fn new<T>(verts: Vec<V>, indices: &[T]) -> Self {
        let mut b = rg::Bindings::default();

        let size_in_bytes = std::mem::size_of::<V>() * verts.len();
        log::trace!("mesh size: {}", size_in_bytes);
        b.vertex_buffers[0] = rg::Buffer::create(&rg::vbuf_desc_dyn(
            size_in_bytes,
            rg::ResourceUsage::Stream,
            "",
        ));

        b.index_buffer = rg::Buffer::create(&rg::ibuf_desc_immutable(indices, ""));

        Self {
            bind: b,
            n_indices: indices.len(),
            verts,
            _phantom: Default::default(),
        }
    }

    /// New mesh with `u16` indices
    pub fn new_16(verts: Vec<V>, indices: &[u16]) -> Self {
        Self::new(verts, indices)
    }

    /// New mesh with `u32` indices
    pub fn new_32(verts: Vec<V>, indices: &[u32]) -> Self {
        Self::new(verts, indices)
    }

    /// slot: [0, 12)
    pub fn bind_image(&mut self, img: rg::Image, slot: usize) {
        self.bind.fs_images[slot] = img;
    }

    /// WARNING: can be called only once a frame
    pub unsafe fn upload_all_verts(&mut self) {
        rg::update_buffer(self.bind.vertex_buffers[0], &self.verts);
        // update_buffer gives us a fresh buffer so make sure we reset our append offset
        self.bind.vertex_buffer_offsets[0] = 0;
    }

    /// WARNING: can be called only once a frame
    ///
    /// * `start_index`: offset for GPU vertex buffer
    pub unsafe fn upload_vert_slice(&mut self, start_index: i32, n_verts: usize) {
        assert!(n_verts <= self.verts.len());
        let start_index = start_index as usize;
        let slice = &self.verts[start_index..start_index + n_verts];
        rg::update_buffer(self.bind.vertex_buffers[0], slice);
    }

    /// Appends vertices to GPU vertex buffer
    ///
    /// * `start_index`: offset for GPU vertex buffer
    pub fn append_vert_slice(&mut self, start_index: i32, n_verts: usize) -> i32 {
        let start_index = start_index as usize;
        debug_assert!(start_index + n_verts <= self.verts.len());

        let slice = &self.verts[start_index..start_index + n_verts];
        let offset = rg::append_buffer(self.bind.vertex_buffers[0], slice);

        // after this: `draw` can be called with `base_elem` being zero
        self.bind.vertex_buffer_offsets[0] = offset;
        offset
    }

    /// Draw call
    ///
    /// Be sure to bind image before calling this.
    ///
    /// `base_elem`: relative to `self.bind.vertex_buffer_offsets[0]`.
    ///
    /// `base_elem` should be zero after calling `append_vert_slice`.
    pub fn draw(&self, base_elem: u32, n_indices: u32) {
        rg::apply_bindings(&self.bind);
        rg::draw(base_elem, n_indices, 1);
    }

    pub fn draw_all(&self) {
        self.draw(0, self.n_indices as u32);
        // self.n_quads = 0;
    }

    // pub fn bindings(&self) -> &rg::Bindings {
    //     &self.bind
    // }
    //
    // pub fn bindings_mut(&mut self) -> &mut rg::Bindings {
    //     &mut self.bind
    // }
}
