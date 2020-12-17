//! SnowRL

pub use {rlbox, rokol, snow2d};

use {
    rokol::gfx as rg,
    snow2d::{
        gfx::{batcher::draw::*, tex::Texture2dDrop},
        Snow2d,
    },
    std::path::PathBuf,
};

use rlbox::{render::tiled as tiled_render, rl::rlmap::TiledRlMap};

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
    rlmap: TiledRlMap,
}

impl RlGame {
    pub fn new() -> Self {
        let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("assets");
        Self {
            rlmap: TiledRlMap::from_tiled_path(&root.join("map/tmx/rl_start.tmx")).unwrap(),
        }
    }

    pub fn update(&mut self) {
        //
    }

    pub fn render(&mut self, rdr: &mut Snow2d) {
        let mut batch = rdr.begin_default_pass();

        tiled_render::render_tiled(
            &mut batch,
            &self.rlmap.tiled,
            &self.rlmap.idmap,
            [(0.0, 0.0), (1280.0, 720.0)],
        );
    }
}
