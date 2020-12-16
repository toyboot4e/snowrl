use {
    glam::{Mat3, Vec2},
    rokol::{
        app as ra,
        gfx::{self as rg, BakedResource, Buffer, Pipeline},
    },
    snow_rl::gfx::{batcher::Batch, texture::TextureData2d},
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

    let mut app = Snow::new();

    rokol.run(&mut app)
}

#[derive(Debug)]
pub struct Snow {
    renderer: snow_rl::Snow2d,
    //
    tex_1: TextureData2d,
    tex_2: TextureData2d,
}

impl Snow {
    pub fn new() -> Self {
        Self {
            renderer: snow_rl::Snow2d::new(),
            tex_1: Default::default(),
            tex_2: Default::default(),
        }
    }
}

impl rokol::app::RApp for Snow {
    fn init(&mut self) {
        rg::setup(&mut rokol::glue::app_desc());

        unsafe {
            self.renderer.init();
        }

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
    }

    fn frame(&mut self) {
        self.update();
        self.render();
        rg::commit();
    }
}

impl Snow {
    pub fn update(&mut self) {
        //
    }

    pub fn render(&mut self) {
        self.renderer.begin_default_pass();

        let white = [255, 255, 255, 255];

        self.renderer.batch.mesh_mut().bind_image(self.tex_1.img, 0);

        self.renderer.batch.push_quad([
            ([200.0, 200.0], white, [0.0, 0.0]).into(),
            ([400.0, 200.0], white, [1.0, 0.0]).into(),
            ([200.0, 400.0], white, [0.0, 1.0]).into(),
            ([400.0, 400.0], white, [1.0, 1.0]).into(),
        ]);

        // let proj = glam::Mat4::identity();
        // unsafe {
        //     rg::apply_uniforms_as_bytes(rg::ShaderStage::Vs, 0, &proj);
        // }

        // self.batch.push_quad([
        //     // pos, color, uv
        //     ([-0.5, 0.5], white, [0.0, 0.0]).into(),
        //     ([0.0, 0.5], white, [1.0, 0.0]).into(),
        //     ([-0.5, -0.5], white, [0.0, 1.0]).into(),
        //     ([0.0, -0.5], white, [1.0, 1.0]).into(),
        // ]);

        self.renderer.end_pass();
    }
}
