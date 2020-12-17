//! SnowRL

use {
    rokol::gfx as rg,
    snow2d::{
        gfx::{batcher::draw::*, tex::Texture2dDrop},
        Snow2d,
    },
    std::path::PathBuf,
};

use rlbox::{render::tiled as tiled_render, rl::rlmap::TiledRlMap};

pub use rlbox;
pub use snow2d;

pub fn run(app: rokol::Rokol) -> rokol::Result {
    app.run(&mut SnowRl::new())
}

#[derive(Debug)]
pub struct SnowRl {
    renderer: Snow2d,
    rl: Option<RlGame>,
}

impl SnowRl {
    pub fn new() -> Self {
        Self {
            renderer: Snow2d::new(),
            rl: None,
        }
    }
}

impl rokol::app::RApp for SnowRl {
    fn init(&mut self) {
        rg::setup(&mut rokol::glue::app_desc());

        unsafe {
            self.renderer.init();
        }

        self.rl = Some(RlGame::new());
    }

    fn frame(&mut self) {
        self.rl.as_mut().unwrap().update();
        self.rl.as_mut().unwrap().render(&mut self.renderer);
        rg::commit();
    }
}

#[derive(Debug)]
pub struct RlGame {
    tex_1: Texture2dDrop,
    tex_2: Texture2dDrop,
    map: TiledRlMap,
}

impl RlGame {
    pub fn new() -> Self {
        let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("assets");
        Self {
            tex_1: {
                let path = root.join("nekura/map2/m_snow02.png");
                Texture2dDrop::from_path(&path).unwrap()
            },

            tex_2: {
                let path = root.join("nekura/map2/m_skelcave.png");
                Texture2dDrop::from_path(&path).unwrap()
            },

            map: TiledRlMap::from_tiled_path(&root.join("map/tmx/rl_start.tmx")).unwrap(),
        }
    }

    pub fn update(&mut self) {
        //
    }

    pub fn render(&mut self, rdr: &mut Snow2d) {
        let mut batch = rdr.begin_default_pass();
        batch.sprite(&self.tex_1).dst_pos_px([400.0, 300.0]);
        batch.sprite(&self.tex_2).dst_pos_px([600.0, 300.0]);
    }
}
