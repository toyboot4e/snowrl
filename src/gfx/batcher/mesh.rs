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

        let size = std::mem::size_of::<V>() * verts.len();
        b.vertex_buffers[0] = rg::Buffer::create(&rg::vbuf_desc_dyn(
            size as i32,
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

    /// Can be called only once a frame
    pub unsafe fn upload_all_verts(&mut self) {
        rg::update_buffer(self.bind.vertex_buffers[0], &self.verts);
        // updateBuffer gives us a fresh buffer so make sure we reset our append offset
        self.bind.vertex_buffer_offsets[0] = 0;
    }

    /// Can be called only once a frame
    pub unsafe fn upload_vert_slice(&mut self, n_verts: usize) {
        assert!(n_verts <= self.verts.len());
        rg::update_buffer(self.bind.vertex_buffers[0], &self.verts[0..n_verts]);
    }

    pub fn append_vert_slice(&mut self, start_index: usize, n_verts: usize) {
        assert!(start_index + n_verts <= self.verts.len());
        let slice = &self.verts[start_index..start_index + n_verts];
        self.bind.vertex_buffer_offsets[0] = rg::append_buffer(self.bind.vertex_buffers[0], slice);
    }

    /// DrawCall
    ///
    /// `base_elem`: relative to `self.bind.vertex_buffer_offsets[0]`. It's zero after calling
    /// `append_vert_slice`.
    pub fn draw(&self, base_elem: u32, n_quads: u32) {
        rg::apply_bindings(&self.bind);
        rg::draw(base_elem, n_quads, 1);
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
