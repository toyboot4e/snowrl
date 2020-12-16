//! SnowRL

use {
    rokol::gfx as rg,
    snow2d::{
        gfx::{batcher::draw::*, texture::TextureData2d},
        Snow2d,
    },
    std::path::PathBuf,
};

#[derive(Debug)]
pub struct SnowRl {
    renderer: Snow2d,
    //
    tex_1: TextureData2d,
    tex_2: TextureData2d,
}

impl SnowRl {
    pub fn new() -> Self {
        Self {
            renderer: Snow2d::new(),
            tex_1: Default::default(),
            tex_2: Default::default(),
        }
    }
}

impl rokol::app::RApp for SnowRl {
    fn init(&mut self) {
        rg::setup(&mut rokol::glue::app_desc());

        unsafe {
            self.renderer.init();
        }

        let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());

        self.tex_1 = {
            let path = root.join("assets/nekura/map2/m_snow02.png");
            TextureData2d::from_path(&path).unwrap()
        };

        self.tex_2 = {
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

impl SnowRl {
    pub fn update(&mut self) {
        //
    }

    pub fn render(&mut self) {
        let mut batch = self.renderer.begin_default_pass();
        batch.sprite(&self.tex_1).dst_pos_px([400.0, 300.0]);
    }
}
