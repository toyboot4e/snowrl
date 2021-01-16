use {
    rokol::gfx::{self as rg, BakedResource},
    std::marker::PhantomData,
};

/// Fluent API to [`rokol::gfx::Bindings`], i.e, vertex/index buffers and image slots
#[derive(Debug, Clone, Default)]
pub struct DynamicMesh<V> {
    pub bind: rg::Bindings,
    pub n_indices: usize,
    pub verts: Vec<V>,
    _phantom: PhantomData<V>,
}

impl<V> DynamicMesh<V> {
    fn new<T>(verts: Vec<V>, indices: &[T]) -> Self {
        let mut b = rg::Bindings::default();

        let size_in_bytes = std::mem::size_of::<V>() * verts.len();
        b.vertex_buffers[0] = rg::Buffer::create(&rg::vbuf_desc_dyn(
            size_in_bytes as i32,
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

    pub fn new_16(verts: Vec<V>, indices: &[u16]) -> Self {
        Self::new(verts, indices)
    }

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
        // updateBuffer gives us a fresh buffer so make sure we reset our append offset
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
        assert!(start_index + n_verts <= self.verts.len());

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
    pub fn draw(&self, base_elem: u32, n_verts: u32) {
        rg::apply_bindings(&self.bind);
        rg::draw(base_elem, n_verts, 1);
        // set `self.mesh.bind.vertex_buffer_offsets[0] = 0;
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