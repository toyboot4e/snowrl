use {
    glam::{Mat3, Vec2},
    rokol::{
        app as ra,
        gfx::{self as rg, BakedResource, Buffer, Pipeline},
    },
    snow_rl::gfx::{batch::Batch, texture::TextureData2d},
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

#[derive(Debug)]
pub struct SnowRl {
    /// Clears the frame color buffer on starting screen rendering pass
    pa: rg::PassAction,
    /// Vertex layouts, shader and render states
    pip: rg::Pipeline,
    /// Vertex/index buffer and images slots
    batch: Batch,
    //
    tex_1: TextureData2d,
    tex_2: TextureData2d,
}

impl SnowRl {
    pub fn new() -> Self {
        let color = [100.0 / 255.0, 149.0 / 255.0, 237.0 / 255.0, 1.0];

        Self {
            pa: rg::PassAction::clear(color),
            pip: Default::default(),
            batch: Default::default(),
            tex_1: Default::default(),
            tex_2: Default::default(),
        }
    }
}

impl rokol::app::RApp for SnowRl {
    fn init(&mut self) {
        rg::setup(&mut rokol::glue::app_desc());

        self.tex_1 = {
            let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
            let path = root.join("assets/nekura/map2/m_snow02.png");
            TextureData2d::from_path(&path).unwrap()
        };
        self.tex_2 = {
            let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
            let path = root.join("assets/nekura/map2/m_skelcave.png");
            TextureData2d::from_path(&path).unwrap()
        };

        self.batch.init();

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

            let white = [255, 255, 255, 255];

            // self.batch.begin().sprite(self.image_1,Mat3::

            // self.batch.mesh_mut().bind_image(self.image_1, 0);
            // self.batch.set_quad([
            //     // pos, color, uv
            //     ([-0.5, 0.5], white, [0.0, 0.0]).into(),
            //     ([0.0, 0.5], white, [1.0, 0.0]).into(),
            //     ([-0.5, -0.5], white, [0.0, 1.0]).into(),
            //     ([0.0, -0.5], white, [1.0, 1.0]).into(),
            // ]);
            // self.batch.flush();

            // self.batch.mesh_mut().bind_image(self.image_2, 0);
            // self.batch.set_quad([
            //     // pos, color, uv
            //     ([0.0, 0.5], white, [0.0, 0.0]).into(),
            //     ([0.5, 0.5], white, [1.0, 0.0]).into(),
            //     ([0.0, -0.5], white, [0.0, 1.0]).into(),
            //     ([0.5, -0.5], white, [1.0, 1.0]).into(),
            // ]);
            // self.batch.flush();
        }
        rg::end_pass();
        rg::commit();
    }
}
