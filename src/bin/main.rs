use {
    rokol::{
        app as ra,
        gfx::{self as rg, BakedResource, Buffer, Pipeline},
    },
    snow_rl::gfx::batch::Vertex,
    std::path::PathBuf,
};

fn main() -> rokol::Result {
    env_logger::init();

    let rokol = rokol::Rokol {
        w: 1280,
        h: 720,
        title: "SnowRL".to_string(),
        ..Default::default()
    };

    let mut app = SnowRl::new();

    rokol.run(&mut app)
}

#[derive(Debug, Default)]
struct SnowRl {
    /// Clears the frame color buffer on starting screen rendering pass
    pa: rg::PassAction,
    /// Vertex layouts, shader and render states
    pip: rg::Pipeline,
    /// Vertex/index buffer and image slots
    bind: rg::Bindings,
}

impl SnowRl {
    pub fn new() -> Self {
        let color = [100.0 / 255.0, 149.0 / 255.0, 237.0 / 255.0, 1.0];

        Self {
            pa: rg::PassAction::clear(color),
            ..Default::default()
        }
    }
}

impl rokol::app::RApp for SnowRl {
    fn init(&mut self) {
        rg::setup(&mut rokol::glue::app_desc());

        self.bind.fs_images[0] = {
            let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
            let path = root.join("assets/nekura/map2/m_snow02.png");
            snow_rl::gfx::load_img(&path)
        };

        self.bind.vertex_buffers[0] = Buffer::create({
            let verts: &[Vertex] = &[
                ([-0.5, -0.5], [255, 255, 255, 255], [0.0, 0.0]).into(),
                ([0.5, -0.5], [255, 255, 255, 255], [1.0, 0.0]).into(),
                ([0.5, 0.5], [255, 255, 255, 255], [1.0, 1.0]).into(),
                ([-0.5, 0.5], [255, 255, 255, 255], [0.0, 1.0]).into(),
            ];

            &rg::vbuf_desc(verts, rg::ResourceUsage::Immutable, "batch-vertices")
        });

        // index for with 2 triangles
        self.bind.index_buffer = Buffer::create({
            let indices: &[u16] = &[0, 1, 2, 0, 2, 3];
            &rg::ibuf_desc(indices, rg::ResourceUsage::Immutable, "batch-indices")
        });

        self.pip = Pipeline::create(&rg::PipelineDesc {
            shader: snow_rl::gfx::shaders::tex_1(),
            index_type: rg::IndexType::UInt16 as u32,
            layout: {
                let mut desc = rg::LayoutDesc::default();
                desc.attrs[0].format = rg::VertexFormat::Float2 as u32;
                desc.attrs[1].format = rg::VertexFormat::UByte4N as u32;
                desc.attrs[2].format = rg::VertexFormat::Float2 as u32;
                desc
            },
            ..Default::default()
        });
    }

    fn frame(&mut self) {
        rg::begin_default_pass(&self.pa, ra::width(), ra::height());
        {
            rg::apply_pipeline(self.pip);
            rg::apply_bindings(&self.bind);
            rg::draw(0, 6, 1); // base_elem, n_indices, n_instances
        }
        rg::end_pass();
        rg::commit();
    }
}
